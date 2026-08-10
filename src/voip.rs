//! Bounded-channel adapters between Python media objects and upstream VoIP traits.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3_async_runtimes::into_future_with_locals;
use pyo3_async_runtimes::tokio::get_current_locals;
use tokio::task::JoinHandle;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use tokio::sync::oneshot;
use whatsapp_rust::async_channel::{self, Receiver, Sender};
use whatsapp_rust::voip::{AudioSink, AudioSource, VideoFrame, VideoSink, VideoSource};

const AUDIO_FRAME_BYTES: usize = 960 * 2;
const DEFAULT_CAPACITY: usize = 6;
const AUDIO_FRAME_SAMPLES: usize = 960;

#[pyclass]
pub struct AudioPlayer {
    receiver: Receiver<Vec<i16>>,
    commands: Sender<AudioCommand>,
    task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    state: Arc<AtomicU8>,
    attached: Arc<AtomicBool>,
}

use std::sync::atomic::AtomicU8;

enum AudioCommand {
    Play(PathBuf, PlayMode, oneshot::Sender<Result<(), String>>),
    Pause(oneshot::Sender<Result<(), String>>),
    Resume(oneshot::Sender<Result<(), String>>),
    Stop(oneshot::Sender<Result<(), String>>),
    Enqueue(PathBuf, oneshot::Sender<Result<(), String>>),
    Skip(oneshot::Sender<Result<(), String>>),
    Clear(oneshot::Sender<Result<(), String>>),
    Finished(u64),
    Shutdown,
}

#[derive(Clone, Copy)]
enum PlayMode { Replace, Queue, Interrupt }

const PLAYER_IDLE: u8 = 0;
const PLAYER_PLAYING: u8 = 1;
const PLAYER_PAUSED: u8 = 2;

#[pyclass]
pub struct VideoPlayer {
    receiver: Mutex<Option<Receiver<Vec<u8>>>>,
    task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    stride: u32,
}

#[pymethods]
impl VideoPlayer {
    #[new]
    #[pyo3(signature = (fps=15))]
    fn new(fps: u32) -> PyResult<Self> {
        if fps == 0 || fps > 60 { return Err(pyo3::exceptions::PyValueError::new_err("fps must be between 1 and 60")); }
        Ok(Self { receiver: Mutex::new(None), task: Mutex::new(None), stride: 90_000 / fps })
    }

    /// Use the Rust FFmpeg process bridge to demux MP4/MOV into H.264 Annex-B.
    fn play(&self, path: String) -> PyResult<()> {
        self.stop();
        let (sender, receiver) = async_channel::bounded(3);
        *self.receiver.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("video player lock poisoned"))? = Some(receiver);
        let task = tokio::spawn(async move {
            let mut child = match Command::new("ffmpeg").args(["-hide_banner", "-loglevel", "error", "-i", &path, "-an", "-c:v", "copy", "-bsf:v", "h264_mp4toannexb", "-f", "h264", "pipe:1"]).stdout(std::process::Stdio::piped()).spawn() { Ok(v) => v, Err(_) => return };
            let Some(mut stdout) = child.stdout.take() else { return };
            let mut buffer = vec![0u8; 64 * 1024];
            let mut splitter = wacore::voip::AnnexBAuSplitter::default();
            loop {
                let count = match stdout.read(&mut buffer).await { Ok(0) | Err(_) => break, Ok(n) => n };
                let mut frames = Vec::new();
                splitter.push(&buffer[..count], &mut frames);
                for frame in frames { if sender.send(frame).await.is_err() { return; } }
            }
            if let Some(frame) = splitter.finish() { let _ = sender.send(frame).await; }
            let _ = child.wait().await;
        });
        *self.task.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("video player lock poisoned"))? = Some(task);
        Ok(())
    }

    fn stop(&self) {
        if let Ok(mut task) = self.task.lock() { if let Some(task) = task.take() { task.abort(); } }
        if let Ok(mut receiver) = self.receiver.lock() { receiver.take(); }
    }
}

impl VideoPlayer {
    pub fn source(&self) -> PyResult<PythonVideoSource> {
        let receiver = self.receiver.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("video player lock poisoned"))?.take().ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("call play() before using VideoPlayer"))?;
        Ok(PythonVideoSource { receiver, stride: self.stride })
    }
}

impl Drop for VideoPlayer { fn drop(&mut self) { self.stop(); } }

#[pymethods]
impl AudioPlayer {
    #[new]
    fn new() -> Self {
        let (commands, command_rx) = async_channel::bounded(32);
        let (output_tx, receiver) = async_channel::bounded(DEFAULT_CAPACITY);
        let state = Arc::new(AtomicU8::new(PLAYER_IDLE));
        let manager_state = Arc::clone(&state);
        let task = tokio::spawn(audio_manager(command_rx, commands.clone(), output_tx, manager_state));
        Self { receiver, commands, task: Mutex::new(Some(task)), state, attached: Arc::new(AtomicBool::new(false)) }
    }

    /// Stream a WAV/MP3/OGG file through the Rust decoder as normalized PCM.
    fn play(&self, path: String, mode: Option<String>) -> PyResult<()> {
        let mode = match mode.as_deref().unwrap_or("replace") {
            "replace" => PlayMode::Replace,
            "queue" => PlayMode::Queue,
            "interrupt" => PlayMode::Interrupt,
            value => return Err(pyo3::exceptions::PyValueError::new_err(format!("unknown playback mode: {value}"))),
        };
        self.send_command(|reply| AudioCommand::Play(PathBuf::from(path), mode, reply))
    }

    fn stop(&self) {
        let _ = self.send_command(|reply| AudioCommand::Stop(reply));
    }

    fn pause(&self) {
        let _ = self.send_command(|reply| AudioCommand::Pause(reply));
    }

    fn resume(&self) {
        let _ = self.send_command(|reply| AudioCommand::Resume(reply));
    }

    fn enqueue(&self, path: String) -> PyResult<()> {
        self.send_command(|reply| AudioCommand::Enqueue(PathBuf::from(path), reply))
    }

    fn skip(&self) -> PyResult<()> {
        self.send_command(|reply| AudioCommand::Skip(reply))
    }

    fn clear_queue(&self) -> PyResult<()> {
        self.send_command(|reply| AudioCommand::Clear(reply))
    }



    #[getter]
    fn state(&self) -> &'static str {
        match self.state.load(Ordering::Acquire) { PLAYER_PLAYING => "playing", PLAYER_PAUSED => "paused", _ => "idle" }
    }
}

impl AudioPlayer {
    fn send_command<F>(&self, make: F) -> PyResult<()>
    where F: FnOnce(oneshot::Sender<Result<(), String>>) -> AudioCommand {
        let (reply_tx, mut reply_rx) = oneshot::channel();
        self.commands.try_send(make(reply_tx)).map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("player command queue is closed"))?;
        match reply_rx.try_recv() {
            Ok(Ok(())) | Err(oneshot::error::TryRecvError::Empty) => Ok(()),
            Ok(Err(error)) => Err(pyo3::exceptions::PyRuntimeError::new_err(error)),
            Err(oneshot::error::TryRecvError::Closed) => Err(pyo3::exceptions::PyRuntimeError::new_err("player manager stopped")),
        }
    }
}

impl AudioPlayer {
    pub fn source(&self) -> PyResult<PythonAudioSource> {
        if self.attached.swap(true, Ordering::AcqRel) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err("AudioPlayer is already attached to a call"));
        }
        Ok(PythonAudioSource { receiver: self.receiver.clone() })
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.commands.try_send(AudioCommand::Shutdown);
        if let Ok(mut task) = self.task.lock() { if let Some(task) = task.take() { task.abort(); } }
    }
}

async fn audio_manager(commands: Receiver<AudioCommand>, command_tx: Sender<AudioCommand>, output: Sender<Vec<i16>>, state: Arc<AtomicU8>) {
    let mut current: Option<PathBuf> = None;
    let mut queue: std::collections::VecDeque<PathBuf> = std::collections::VecDeque::new();
    let mut cancel: Option<Arc<AtomicBool>> = None;
    let mut paused: Option<Arc<AtomicBool>> = None;
    let mut decoder: Option<tokio::task::JoinHandle<()>> = None;
    let mut generation: u64 = 0;
    let mut current_generation: Option<u64> = None;
    while let Ok(command) = commands.recv().await {
        match command {
            AudioCommand::Play(path, mode, reply) => {
                if matches!(mode, PlayMode::Queue) && current.is_some() { queue.push_back(path); let _ = reply.send(Ok(())); continue; }
                let interrupted = matches!(mode, PlayMode::Interrupt);
                if interrupted {
                    if let Some(active) = current.clone() { queue.push_front(active); }
                }
                if let Some(flag) = cancel.take() { flag.store(true, Ordering::Release); }
                if let Some(task) = decoder.take() { task.abort(); }
                if !interrupted { queue.clear(); }
                current = Some(path.clone()); state.store(PLAYER_PLAYING, Ordering::Release);
                generation = generation.wrapping_add(1);
                let playback_id = generation;
                let flag = Arc::new(AtomicBool::new(false));
                let paused_gate = Arc::new(AtomicBool::new(false));
                let flag_task = Arc::clone(&flag); let paused_task = Arc::clone(&paused_gate);
                let output_task = output.clone();
                let finish_tx = command_tx.clone();
                decoder = Some(tokio::spawn(async move { let _ = tokio::task::spawn_blocking(move || decode_audio_file(path, output_task, flag_task, paused_task)).await; let _ = finish_tx.send(AudioCommand::Finished(playback_id)).await; }));
                cancel = Some(flag); paused = Some(paused_gate); current_generation = Some(playback_id); let _ = reply.send(Ok(()));
            }
            AudioCommand::Pause(reply) => { if let Some(flag) = &paused { flag.store(true, Ordering::Release); } state.store(PLAYER_PAUSED, Ordering::Release); let _ = reply.send(Ok(())); }
            AudioCommand::Resume(reply) => { if let Some(flag) = &paused { flag.store(false, Ordering::Release); } state.store(PLAYER_PLAYING, Ordering::Release); let _ = reply.send(Ok(())); }
            AudioCommand::Stop(reply) => {
                if let Some(flag) = cancel.take() { flag.store(true, Ordering::Release); }
                if let Some(task) = decoder.take() { task.abort(); }
                current = None; queue.clear(); state.store(PLAYER_IDLE, Ordering::Release); let _ = reply.send(Ok(()));
                paused = None;
                current_generation = None;
            }
            AudioCommand::Enqueue(path, reply) => { queue.push_back(path); let _ = reply.send(Ok(())); }
            AudioCommand::Skip(reply) => {
                if let Some(flag) = cancel.take() { flag.store(true, Ordering::Release); }
                if let Some(task) = decoder.take() { task.abort(); }
                if let Some(path) = queue.pop_front() {
                    current = Some(path.clone()); let flag = Arc::new(AtomicBool::new(false)); let paused_gate = Arc::new(AtomicBool::new(false));
                    let flag_task = Arc::clone(&flag); let paused_task = Arc::clone(&paused_gate);
                    let output_task = output.clone();
                    generation = generation.wrapping_add(1); let playback_id = generation;
                    let finish_tx = command_tx.clone();
                    decoder = Some(tokio::spawn(async move { let _ = tokio::task::spawn_blocking(move || decode_audio_file(path, output_task, flag_task, paused_task)).await; let _ = finish_tx.send(AudioCommand::Finished(playback_id)).await; })); cancel = Some(flag); paused = Some(paused_gate); current_generation = Some(playback_id); state.store(PLAYER_PLAYING, Ordering::Release);
                } else { current = None; state.store(PLAYER_IDLE, Ordering::Release); }
                let _ = reply.send(Ok(()));
            }
            AudioCommand::Clear(reply) => { queue.clear(); let _ = reply.send(Ok(())); }
            AudioCommand::Finished(playback_id) => {
                if current_generation != Some(playback_id) { continue; }
                decoder = None; cancel = None; paused = None;
                if let Some(path) = queue.pop_front() {
                    current = Some(path.clone()); generation = generation.wrapping_add(1); let next_id = generation;
                    let flag = Arc::new(AtomicBool::new(false)); let paused_gate = Arc::new(AtomicBool::new(false));
                    let output_task = output.clone(); let flag_task = Arc::clone(&flag); let paused_task = Arc::clone(&paused_gate);
                    let finish_tx = command_tx.clone();
                    decoder = Some(tokio::spawn(async move { let _ = tokio::task::spawn_blocking(move || decode_audio_file(path, output_task, flag_task, paused_task)).await; let _ = finish_tx.send(AudioCommand::Finished(next_id)).await; }));
                    cancel = Some(flag); paused = Some(paused_gate); current_generation = Some(next_id); state.store(PLAYER_PLAYING, Ordering::Release);
                } else { current = None; current_generation = None; state.store(PLAYER_IDLE, Ordering::Release); }
            }
            AudioCommand::Shutdown => break,
        }
    }
}

fn decode_audio_file(path: PathBuf, sender: Sender<Vec<i16>>, stopped: Arc<AtomicBool>, paused: Arc<AtomicBool>) -> Result<(), String> {
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::default::{get_codecs, get_probe};

    let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) { hint.with_extension(ext); }
    let mut probed = get_probe().format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default()).map_err(|e| e.to_string())?;
    let track = probed.format.default_track().ok_or("audio track not found")?;
    let track_id = track.id;
    let rate = track.codec_params.sample_rate.ok_or("sample rate missing")? as usize;
    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default()).map_err(|e| e.to_string())?;
    let mut carry = Vec::with_capacity(AUDIO_FRAME_SAMPLES * 2);
    loop {
        let packet = match probed.format.next_packet() {
            Ok(p) if p.track_id() == track_id => p,
            Ok(_) => continue,
            Err(Error::IoError(_)) => break,
            Err(Error::ResetRequired) => { decoder.reset(); continue; },
            Err(e) => return Err(e.to_string()),
        };
        let decoded = decoder.decode(&packet).map_err(|e| e.to_string())?;
        let mut samples = Vec::new();
        match decoded {
            AudioBufferRef::U8(v) => samples.extend(v.chan(0).iter().map(|x| (*x as i16 - 128) << 8)),
            AudioBufferRef::U16(v) => samples.extend(v.chan(0).iter().map(|x| (*x as i32 - 32768) as i16)),
            AudioBufferRef::U24(v) => samples.extend(v.chan(0).iter().map(|x| (x.inner() >> 8) as i16)),
            AudioBufferRef::U32(v) => samples.extend(v.chan(0).iter().map(|x| ((*x as i64 - 2147483648) >> 16) as i16)),
            AudioBufferRef::S8(v) => samples.extend(v.chan(0).iter().map(|x| (*x as i16) << 8)),
            AudioBufferRef::S16(v) => samples.extend(v.chan(0).iter().copied()),
            AudioBufferRef::S24(v) => samples.extend(v.chan(0).iter().map(|x| (x.inner() >> 8) as i16)),
            AudioBufferRef::S32(v) => samples.extend(v.chan(0).iter().map(|x| (*x >> 16) as i16)),
            AudioBufferRef::F32(v) => samples.extend(v.chan(0).iter().map(|x| (x.clamp(-1.0, 1.0) * 32767.0) as i16)),
            AudioBufferRef::F64(v) => samples.extend(v.chan(0).iter().map(|x| (x.clamp(-1.0, 1.0) * 32767.0) as i16)),
        }
        let normalized: Vec<i16> = if rate == 16000 { samples } else {
            let count = samples.len() * 16000 / rate;
            (0..count).map(|i| samples[(i * rate / 16000).min(samples.len().saturating_sub(1))]).collect()
        };
        carry.extend(normalized);
        while carry.len() >= AUDIO_FRAME_SAMPLES {
            let frame: Vec<i16> = carry.drain(..AUDIO_FRAME_SAMPLES).collect();
            loop {
                if stopped.load(Ordering::Acquire) { return Ok(()); }
                if paused.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                }
                match sender.try_send(frame.clone()) {
                    Ok(()) => break,
                    Err(async_channel::TrySendError::Full(_)) => std::thread::sleep(Duration::from_millis(2)),
                    Err(async_channel::TrySendError::Closed(_)) => return Ok(()),
                }
            }
        }
    }
    if !carry.is_empty() {
        carry.resize(AUDIO_FRAME_SAMPLES, 0);
        while !stopped.load(Ordering::Acquire) && !paused.load(Ordering::Acquire) {
            match sender.try_send(carry.clone()) {
                Ok(()) | Err(async_channel::TrySendError::Closed(_)) => break,
                Err(async_channel::TrySendError::Full(_)) => std::thread::sleep(Duration::from_millis(2)),
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct PythonAudioSource {
    receiver: Receiver<Vec<i16>>,
}

impl AudioSource for PythonAudioSource {
    fn frames(&self) -> Receiver<Vec<i16>> {
        self.receiver.clone()
    }
}

#[derive(Clone)]
pub struct PythonAudioSink {
    sender: Sender<Vec<i16>>,
}

impl AudioSink for PythonAudioSink {
    fn playout(&self) -> Sender<Vec<i16>> {
        self.sender.clone()
    }
}

#[derive(Clone)]
pub struct PythonVideoSource {
    receiver: Receiver<Vec<u8>>,
    stride: u32,
}

impl VideoSource for PythonVideoSource {
    fn frames(&self) -> Receiver<Vec<u8>> {
        self.receiver.clone()
    }
    fn rtp_timestamp_stride(&self) -> u32 {
        self.stride
    }
}

#[derive(Clone)]
pub struct PythonVideoSink {
    sender: Sender<VideoFrame>,
}

impl VideoSink for PythonVideoSink {
    fn playout(&self) -> Sender<VideoFrame> {
        self.sender.clone()
    }
}

pub struct PythonMediaBridge {
    pub audio_source: Option<PythonAudioSource>,
    pub audio_sink: Option<PythonAudioSink>,
    pub video_source: Option<PythonVideoSource>,
    pub video_sink: Option<PythonVideoSink>,
    pub tasks: Vec<JoinHandle<()>>,
}

fn pcm16_from_python(value: Bound<'_, PyAny>) -> PyResult<Vec<i16>> {
    let bytes = value.extract::<Vec<u8>>()?;
    if bytes.len() != AUDIO_FRAME_BYTES {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "audio frame must be exactly {AUDIO_FRAME_BYTES} bytes (960 PCM16 samples), got {}",
            bytes.len()
        )));
    }
    Ok(bytes
        .chunks_exact(2)
        .map(|v| i16::from_le_bytes([v[0], v[1]]))
        .collect())
}

fn pcm16_to_python(py: Python<'_>, frame: &[i16]) -> Py<PyAny> {
    let mut bytes = Vec::with_capacity(frame.len() * 2);
    for sample in frame {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    pyo3::types::PyBytes::new(py, &bytes).unbind().into_any()
}

pub fn audio_source_from_python(
    py: Python<'_>,
    source: Py<PyAny>,
    capacity: usize,
) -> PyResult<(PythonAudioSource, JoinHandle<()>)> {
    let iterator = source.call_method0(py, "frames")?;
    let locals = get_current_locals(py)?;
    let (sender, receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) =
        async_channel::bounded(capacity.max(1));
    let task = tokio::spawn(async move {
        loop {
            let next = Python::attach(|py| {
                let awaitable = iterator.call_method0(py, "__anext__")?;
                into_future_with_locals(&locals, awaitable.into_bound(py))
            });
            let Ok(next) = next else { break };
            let Ok(value) = next.await else { break };
            let frame = Python::attach(|py| pcm16_from_python(value.bind(py).clone()));
            let Ok(frame) = frame else { break };
            if sender.send(frame).await.is_err() {
                break;
            }
        }
    });
    Ok((PythonAudioSource { receiver }, task))
}

pub fn audio_sink_from_python(
    py: Python<'_>,
    sink: Py<PyAny>,
    capacity: usize,
) -> PyResult<(PythonAudioSink, JoinHandle<()>)> {
    let locals = get_current_locals(py)?;
    let (sender, receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) =
        async_channel::bounded(capacity.max(1));
    let task = tokio::spawn(async move {
        while let Ok(frame) = receiver.recv().await {
            let awaitable = Python::attach(|py| {
                let bytes = pcm16_to_python(py, &frame);
                sink.call_method1(py, "write", (bytes,))
            });
            let Ok(awaitable) = awaitable else { break };
            let Ok(future) =
                Python::attach(|py| into_future_with_locals(&locals, awaitable.into_bound(py)))
            else {
                break;
            };
            if future.await.is_err() {
                break;
            }
        }
    });
    Ok((PythonAudioSink { sender }, task))
}

pub fn video_source_from_python(
    py: Python<'_>,
    source: Py<PyAny>,
    capacity: usize,
    stride: u32,
) -> PyResult<(PythonVideoSource, JoinHandle<()>)> {
    let iterator = source.call_method0(py, "frames")?;
    let locals = get_current_locals(py)?;
    let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) =
        async_channel::bounded(capacity.max(1));
    let task = tokio::spawn(async move {
        loop {
            let next = Python::attach(|py| {
                let awaitable = match iterator.call_method0(py, "__anext__") {
                    Ok(value) => value,
                    Err(_) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(
                            "video source ended",
                        ));
                    }
                };
                into_future_with_locals(&locals, awaitable.into_bound(py))
            });
            let Ok(next) = next else { break };
            let Ok(value) = next.await else { break };
            let frame = Python::attach(|py| {
                let data: Vec<u8> = value.bind(py).getattr("data")?.extract()?;
                if data.is_empty() {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "video frame cannot be empty",
                    ));
                }
                Ok(data)
            });
            let Ok(frame) = frame else { break };
            if sender.send(frame).await.is_err() {
                break;
            }
        }
    });
    Ok((
        PythonVideoSource {
            receiver,
            stride: stride.max(1),
        },
        task,
    ))
}

pub fn video_sink_from_python(
    py: Python<'_>,
    sink: Py<PyAny>,
    capacity: usize,
) -> PyResult<(PythonVideoSink, JoinHandle<()>)> {
    let locals = get_current_locals(py)?;
    let (sender, receiver): (Sender<VideoFrame>, Receiver<VideoFrame>) =
        async_channel::bounded(capacity.max(1));
    let task = tokio::spawn(async move {
        while let Ok(frame) = receiver.recv().await {
            let result = Python::attach(|py| {
                let media = py.import("tryx.media")?;
                let frame_type = media.getattr("VideoFrame")?;
                let value = frame_type.call1((
                    frame.data,
                    py.None(),
                    66_666_i64,
                    frame.keyframe,
                    py.None(),
                    py.None(),
                    frame.orientation,
                ))?;
                let awaitable = sink.call_method1(py, "write", (value,))?;
                into_future_with_locals(&locals, awaitable.into_bound(py))
            });
            let Ok(future) = result else { break };
            if future.await.is_err() {
                break;
            }
        }
    });
    Ok((PythonVideoSink { sender }, task))
}

pub fn audio_channels(
    capacity: usize,
) -> (
    PythonAudioSource,
    Sender<Vec<i16>>,
    PythonAudioSink,
    Receiver<Vec<i16>>,
) {
    let (source_tx, source_rx) = async_channel::bounded(capacity.max(1));
    let (sink_tx, sink_rx) = async_channel::bounded(capacity.max(1));
    (
        PythonAudioSource {
            receiver: source_rx,
        },
        source_tx,
        PythonAudioSink { sender: sink_tx },
        sink_rx,
    )
}

pub fn video_channels(
    capacity: usize,
) -> (
    PythonVideoSource,
    Sender<Vec<u8>>,
    PythonVideoSink,
    Receiver<VideoFrame>,
) {
    let (source_tx, source_rx) = async_channel::bounded(capacity.max(1));
    let (sink_tx, sink_rx) = async_channel::bounded(capacity.max(1));
    (
        PythonVideoSource {
            receiver: source_rx,
            stride: 90_000 / 15,
        },
        source_tx,
        PythonVideoSink { sender: sink_tx },
        sink_rx,
    )
}

pub fn bridge_from_python(
    py: Python<'_>,
    audio_source: Option<Py<PyAny>>,
    audio_sink: Option<Py<PyAny>>,
    video_source: Option<Py<PyAny>>,
    video_sink: Option<Py<PyAny>>,
) -> PyResult<PythonMediaBridge> {
    let mut bridge = PythonMediaBridge {
        audio_source: None,
        audio_sink: None,
        video_source: None,
        video_sink: None,
        tasks: Vec::new(),
    };
    if let Some(source) = audio_source {
        let native = source.extract::<Py<AudioPlayer>>(py);
        let (adapter, task) = if let Ok(player) = native {
            let adapter = player.borrow(py).source()?;
            (adapter, tokio::spawn(async {}))
        } else {
            audio_source_from_python(py, source, DEFAULT_CAPACITY)?
        };
        bridge.audio_source = Some(adapter);
        bridge.tasks.push(task);
    }
    if let Some(sink) = audio_sink {
        let (adapter, task) = audio_sink_from_python(py, sink, DEFAULT_CAPACITY)?;
        bridge.audio_sink = Some(adapter);
        bridge.tasks.push(task);
    }
    if let Some(source) = video_source {
        let native = source.extract::<Py<VideoPlayer>>(py);
        let (adapter, task) = if let Ok(player) = native {
            (player.borrow(py).source()?, tokio::spawn(async {}))
        } else {
            video_source_from_python(py, source, 3, 90_000 / 15)?
        };
        bridge.video_source = Some(adapter); bridge.tasks.push(task);
    }
    if let Some(sink) = video_sink {
        let (adapter, task) = video_sink_from_python(py, sink, 3)?;
        bridge.video_sink = Some(adapter); bridge.tasks.push(task);
    }
    Ok(bridge)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_adapters_are_bounded() {
        let (_source, tx, _sink, _rx) = audio_channels(2);
        assert_eq!(tx.capacity(), Some(2));
    }

    #[test]
    fn playback_modes_are_explicit() {
        assert!(matches!(PlayMode::Replace, PlayMode::Replace));
        assert!(matches!(PlayMode::Queue, PlayMode::Queue));
        assert!(matches!(PlayMode::Interrupt, PlayMode::Interrupt));
    }
}
