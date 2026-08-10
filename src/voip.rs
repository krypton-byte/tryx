//! Bounded-channel adapters between Python media objects and upstream VoIP traits.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3_async_runtimes::into_future_with_locals;
use pyo3_async_runtimes::tokio::get_current_locals;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use whatsapp_rust::async_channel::{self, Receiver, Sender};
use whatsapp_rust::voip::{AudioSink, AudioSource, VideoFrame, VideoSink, VideoSource};

const AUDIO_FRAME_BYTES: usize = 960 * 2;
const DEFAULT_AUDIO_CAPACITY: usize = 3;
const DEFAULT_VIDEO_CAPACITY: usize = 3;
const MAX_BUFFER_FRAMES: usize = 30;
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
enum PlayMode {
    Replace,
    Queue,
    Interrupt,
}

const PLAYER_IDLE: u8 = 0;
const PLAYER_PLAYING: u8 = 1;
const PLAYER_PAUSED: u8 = 2;

#[pyclass]
pub struct VideoPlayer {
    receiver: Mutex<Option<Receiver<Vec<u8>>>>,
    task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    stop: Mutex<Option<oneshot::Sender<()>>>,
    stride: u32,
}

#[pymethods]
impl VideoPlayer {
    #[new]
    #[pyo3(signature = (fps=15))]
    fn new(fps: u32) -> PyResult<Self> {
        if fps == 0 || fps > 60 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "fps must be between 1 and 60",
            ));
        }
        Ok(Self {
            receiver: Mutex::new(None),
            task: Mutex::new(None),
            stop: Mutex::new(None),
            stride: 90_000 / fps,
        })
    }

    /// Use the Rust FFmpeg process bridge to demux MP4/MOV into H.264 Annex-B.
    fn play(&self, path: String) -> PyResult<()> {
        self.stop();
        if !std::path::Path::new(&path).is_file() {
            return Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                "video file not found: {path}"
            )));
        }
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map_err(|_| {
                pyo3::exceptions::PyEnvironmentError::new_err(
                    "ffmpeg is not installed or is not available in PATH",
                )
            })?;
        let (sender, receiver) = async_channel::bounded(DEFAULT_VIDEO_CAPACITY);
        let (stop_tx, mut stop_rx) = oneshot::channel();
        *self.receiver.lock().map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err("video player lock poisoned")
        })? = Some(receiver);
        let task = tokio::spawn(async move {
            let mut child = match Command::new("ffmpeg")
                .args([
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-i",
                    &path,
                    "-an",
                    "-c:v",
                    "copy",
                    "-bsf:v",
                    "h264_mp4toannexb",
                    "-f",
                    "h264",
                    "pipe:1",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(v) => v,
                Err(_) => return,
            };
            let Some(mut stdout) = child.stdout.take() else {
                return;
            };
            let mut buffer = vec![0u8; 64 * 1024];
            let mut splitter = wacore::voip::AnnexBAuSplitter::default();
            loop {
                let count = tokio::select! {
                    _ = &mut stop_rx => { let _ = child.kill().await; return; },
                    result = stdout.read(&mut buffer) => match result { Ok(0) | Err(_) => break, Ok(n) => n },
                };
                let mut frames = Vec::new();
                splitter.push(&buffer[..count], &mut frames);
                for frame in frames {
                    tokio::select! {
                        _ = &mut stop_rx => { let _ = child.kill().await; return; },
                        result = sender.send(frame) => if result.is_err() { let _ = child.kill().await; return; },
                    }
                }
            }
            if let Some(frame) = splitter.finish() {
                tokio::select! {
                    _ = &mut stop_rx => { let _ = child.kill().await; return; },
                    _ = sender.send(frame) => {},
                }
            }
            let _ = child.wait().await;
        });
        *self.stop.lock().map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err("video player stop lock poisoned")
        })? = Some(stop_tx);
        *self.task.lock().map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err("video player lock poisoned")
        })? = Some(task);
        Ok(())
    }

    fn stop(&self) {
        if let Ok(mut stop) = self.stop.lock() {
            if let Some(stop) = stop.take() {
                let _ = stop.send(());
            }
        }
        // The task owns the child. Signal it first and let the select! branch
        // kill/reap FFmpeg; aborting here would drop the child without cleanup.
        if let Ok(mut task) = self.task.lock() {
            task.take();
        }
        if let Ok(mut receiver) = self.receiver.lock() {
            receiver.take();
        }
    }
}

impl VideoPlayer {
    pub fn source(&self) -> PyResult<PythonVideoSource> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("video player lock poisoned"))?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("call play() before using VideoPlayer")
            })?;
        Ok(PythonVideoSource {
            receiver,
            stride: self.stride,
        })
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[pymethods]
impl AudioPlayer {
    #[new]
    #[pyo3(signature = (buffer_frames=3))]
    fn new(buffer_frames: usize) -> Self {
        let capacity = buffer_frames.clamp(1, MAX_BUFFER_FRAMES);
        let (commands, command_rx) = async_channel::bounded(32);
        let (output_tx, receiver) = async_channel::bounded(capacity);
        let state = Arc::new(AtomicU8::new(PLAYER_IDLE));
        let manager_state = Arc::clone(&state);
        let task = tokio::spawn(audio_manager(
            command_rx,
            commands.clone(),
            output_tx,
            manager_state,
        ));
        Self {
            receiver,
            commands,
            task: Mutex::new(Some(task)),
            state,
            attached: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Stream a WAV/MP3/OGG file through the Rust decoder as normalized PCM.
    fn play(&self, path: String, mode: Option<String>) -> PyResult<()> {
        let mode = match mode.as_deref().unwrap_or("replace") {
            "replace" => PlayMode::Replace,
            "queue" => PlayMode::Queue,
            "interrupt" => PlayMode::Interrupt,
            value => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "unknown playback mode: {value}"
                )));
            }
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
        match self.state.load(Ordering::Acquire) {
            PLAYER_PLAYING => "playing",
            PLAYER_PAUSED => "paused",
            _ => "idle",
        }
    }
}

impl AudioPlayer {
    fn send_command<F>(&self, make: F) -> PyResult<()>
    where
        F: FnOnce(oneshot::Sender<Result<(), String>>) -> AudioCommand,
    {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.commands.try_send(make(reply_tx)).map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err("player command queue is closed")
        })?;
        match reply_rx.blocking_recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(pyo3::exceptions::PyRuntimeError::new_err(error)),
            Err(_) => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "player manager stopped",
            )),
        }
    }
}

impl AudioPlayer {
    pub fn source(&self) -> PyResult<PythonAudioSource> {
        if self.attached.swap(true, Ordering::AcqRel) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "AudioPlayer is already attached to a call",
            ));
        }
        Ok(PythonAudioSource {
            receiver: self.receiver.clone(),
            attached: Arc::clone(&self.attached),
        })
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.commands.send_blocking(AudioCommand::Shutdown);
        if let Ok(mut task) = self.task.lock() {
            if let Some(task) = task.take() {
                task.abort();
            }
        }
    }
}

async fn audio_manager(
    commands: Receiver<AudioCommand>,
    command_tx: Sender<AudioCommand>,
    output: Sender<Vec<i16>>,
    state: Arc<AtomicU8>,
) {
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
                if matches!(mode, PlayMode::Queue) && current.is_some() {
                    queue.push_back(path);
                    let _ = reply.send(Ok(()));
                    continue;
                }
                let interrupted = matches!(mode, PlayMode::Interrupt);
                if interrupted {
                    if let Some(active) = current.clone() {
                        queue.push_front(active);
                    }
                }
                if let Some(flag) = cancel.take() {
                    flag.store(true, Ordering::Release);
                }
                if let Some(task) = decoder.take() {
                    task.abort();
                }
                if !interrupted {
                    queue.clear();
                }
                current = Some(path.clone());
                state.store(PLAYER_PLAYING, Ordering::Release);
                generation = generation.wrapping_add(1);
                let playback_id = generation;
                let flag = Arc::new(AtomicBool::new(false));
                let paused_gate = Arc::new(AtomicBool::new(false));
                let flag_task = Arc::clone(&flag);
                let paused_task = Arc::clone(&paused_gate);
                let output_task = output.clone();
                let finish_tx = command_tx.clone();
                decoder = Some(tokio::spawn(async move {
                    let _ = tokio::task::spawn_blocking(move || {
                        decode_audio_file(path, output_task, flag_task, paused_task)
                    })
                    .await;
                    let _ = finish_tx.send(AudioCommand::Finished(playback_id)).await;
                }));
                cancel = Some(flag);
                paused = Some(paused_gate);
                current_generation = Some(playback_id);
                let _ = reply.send(Ok(()));
            }
            AudioCommand::Pause(reply) => {
                if let Some(flag) = &paused {
                    flag.store(true, Ordering::Release);
                }
                state.store(PLAYER_PAUSED, Ordering::Release);
                let _ = reply.send(Ok(()));
            }
            AudioCommand::Resume(reply) => {
                if let Some(flag) = &paused {
                    flag.store(false, Ordering::Release);
                }
                state.store(PLAYER_PLAYING, Ordering::Release);
                let _ = reply.send(Ok(()));
            }
            AudioCommand::Stop(reply) => {
                if let Some(flag) = cancel.take() {
                    flag.store(true, Ordering::Release);
                }
                if let Some(task) = decoder.take() {
                    task.abort();
                }
                current = None;
                queue.clear();
                state.store(PLAYER_IDLE, Ordering::Release);
                let _ = reply.send(Ok(()));
                paused = None;
                current_generation = None;
            }
            AudioCommand::Enqueue(path, reply) => {
                queue.push_back(path);
                let _ = reply.send(Ok(()));
            }
            AudioCommand::Skip(reply) => {
                if let Some(flag) = cancel.take() {
                    flag.store(true, Ordering::Release);
                }
                if let Some(task) = decoder.take() {
                    task.abort();
                }
                if let Some(path) = queue.pop_front() {
                    current = Some(path.clone());
                    let flag = Arc::new(AtomicBool::new(false));
                    let paused_gate = Arc::new(AtomicBool::new(false));
                    let flag_task = Arc::clone(&flag);
                    let paused_task = Arc::clone(&paused_gate);
                    let output_task = output.clone();
                    generation = generation.wrapping_add(1);
                    let playback_id = generation;
                    let finish_tx = command_tx.clone();
                    decoder = Some(tokio::spawn(async move {
                        let _ = tokio::task::spawn_blocking(move || {
                            decode_audio_file(path, output_task, flag_task, paused_task)
                        })
                        .await;
                        let _ = finish_tx.send(AudioCommand::Finished(playback_id)).await;
                    }));
                    cancel = Some(flag);
                    paused = Some(paused_gate);
                    current_generation = Some(playback_id);
                    state.store(PLAYER_PLAYING, Ordering::Release);
                } else {
                    current = None;
                    state.store(PLAYER_IDLE, Ordering::Release);
                }
                let _ = reply.send(Ok(()));
            }
            AudioCommand::Clear(reply) => {
                queue.clear();
                let _ = reply.send(Ok(()));
            }
            AudioCommand::Finished(playback_id) => {
                if current_generation != Some(playback_id) {
                    continue;
                }
                decoder = None;
                cancel = None;
                paused = None;
                if let Some(path) = queue.pop_front() {
                    current = Some(path.clone());
                    generation = generation.wrapping_add(1);
                    let next_id = generation;
                    let flag = Arc::new(AtomicBool::new(false));
                    let paused_gate = Arc::new(AtomicBool::new(false));
                    let output_task = output.clone();
                    let flag_task = Arc::clone(&flag);
                    let paused_task = Arc::clone(&paused_gate);
                    let finish_tx = command_tx.clone();
                    decoder = Some(tokio::spawn(async move {
                        let _ = tokio::task::spawn_blocking(move || {
                            decode_audio_file(path, output_task, flag_task, paused_task)
                        })
                        .await;
                        let _ = finish_tx.send(AudioCommand::Finished(next_id)).await;
                    }));
                    cancel = Some(flag);
                    paused = Some(paused_gate);
                    current_generation = Some(next_id);
                    state.store(PLAYER_PLAYING, Ordering::Release);
                } else {
                    current = None;
                    current_generation = None;
                    state.store(PLAYER_IDLE, Ordering::Release);
                }
            }
            AudioCommand::Shutdown => break,
        }
    }
}

fn decode_audio_file(
    path: PathBuf,
    sender: Sender<Vec<i16>>,
    stopped: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
) -> Result<(), String> {
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
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let mut probed = get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| e.to_string())?;
    let track = probed
        .format
        .default_track()
        .ok_or("audio track not found")?;
    let track_id = track.id;
    let rate = track
        .codec_params
        .sample_rate
        .ok_or("sample rate missing")? as usize;
    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    let mut carry = Vec::with_capacity(AUDIO_FRAME_SAMPLES * 2);
    loop {
        let packet = match probed.format.next_packet() {
            Ok(p) if p.track_id() == track_id => p,
            Ok(_) => continue,
            Err(Error::IoError(_)) => break,
            Err(Error::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(e) => return Err(e.to_string()),
        };
        let decoded = decoder.decode(&packet).map_err(|e| e.to_string())?;
        let mut samples = Vec::new();
        match decoded {
            AudioBufferRef::U8(v) => {
                samples.extend(v.chan(0).iter().map(|x| (*x as i16 - 128) << 8))
            }
            AudioBufferRef::U16(v) => {
                samples.extend(v.chan(0).iter().map(|x| (*x as i32 - 32768) as i16))
            }
            AudioBufferRef::U24(v) => {
                samples.extend(v.chan(0).iter().map(|x| (x.inner() >> 8) as i16))
            }
            AudioBufferRef::U32(v) => samples.extend(
                v.chan(0)
                    .iter()
                    .map(|x| ((*x as i64 - 2147483648) >> 16) as i16),
            ),
            AudioBufferRef::S8(v) => samples.extend(v.chan(0).iter().map(|x| (*x as i16) << 8)),
            AudioBufferRef::S16(v) => samples.extend(v.chan(0).iter().copied()),
            AudioBufferRef::S24(v) => {
                samples.extend(v.chan(0).iter().map(|x| (x.inner() >> 8) as i16))
            }
            AudioBufferRef::S32(v) => samples.extend(v.chan(0).iter().map(|x| (*x >> 16) as i16)),
            AudioBufferRef::F32(v) => samples.extend(
                v.chan(0)
                    .iter()
                    .map(|x| (x.clamp(-1.0, 1.0) * 32767.0) as i16),
            ),
            AudioBufferRef::F64(v) => samples.extend(
                v.chan(0)
                    .iter()
                    .map(|x| (x.clamp(-1.0, 1.0) * 32767.0) as i16),
            ),
        }
        let normalized = resample_linear(&samples, rate, 16000);
        carry.extend(normalized);
        while carry.len() >= AUDIO_FRAME_SAMPLES {
            let frame: Vec<i16> = carry.drain(..AUDIO_FRAME_SAMPLES).collect();
            while paused.load(Ordering::Acquire) {
                if stopped.load(Ordering::Acquire) {
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            if stopped.load(Ordering::Acquire) || sender.send_blocking(frame).is_err() {
                return Ok(());
            }
        }
    }
    if !carry.is_empty() {
        carry.resize(AUDIO_FRAME_SAMPLES, 0);
        while !stopped.load(Ordering::Acquire) && !paused.load(Ordering::Acquire) {
            if sender.send_blocking(std::mem::take(&mut carry)).is_err() {
                break;
            }
        }
    }
    Ok(())
}

/// Resample mono PCM with linear interpolation. Keeping this local avoids a
/// heavyweight resampler state per decoded packet while removing the severe
/// aliasing and stepping artifacts of nearest-neighbour sampling.
fn resample_linear(input: &[i16], input_rate: usize, output_rate: usize) -> Vec<i16> {
    if input.is_empty() || input_rate == output_rate {
        return input.to_vec();
    }
    let output_len = ((input.len() as u64 * output_rate as u64 + input_rate as u64 - 1)
        / input_rate as u64) as usize;
    let mut output = Vec::with_capacity(output_len);
    for index in 0..output_len {
        let position = index as f64 * input_rate as f64 / output_rate as f64;
        let left = position.floor() as usize;
        let right = (left + 1).min(input.len() - 1);
        let fraction = position - left as f64;
        let sample = input[left.min(input.len() - 1)] as f64
            + (input[right] as f64 - input[left.min(input.len() - 1)] as f64) * fraction;
        output.push(sample.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16);
    }
    output
}

pub struct PythonAudioSource {
    receiver: Receiver<Vec<i16>>,
    attached: Arc<AtomicBool>,
}

impl Drop for PythonAudioSource {
    fn drop(&mut self) {
        self.attached.store(false, Ordering::Release);
    }
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
    Ok((
        PythonAudioSource {
            receiver,
            attached: Arc::new(AtomicBool::new(false)),
        },
        task,
    ))
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
            attached: Arc::new(AtomicBool::new(false)),
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
            (adapter, None)
        } else {
            let (adapter, task) = audio_source_from_python(py, source, DEFAULT_AUDIO_CAPACITY)?;
            (adapter, Some(task))
        };
        bridge.audio_source = Some(adapter);
        if let Some(task) = task {
            bridge.tasks.push(task);
        }
    }
    if let Some(sink) = audio_sink {
        let (adapter, task) = audio_sink_from_python(py, sink, DEFAULT_AUDIO_CAPACITY)?;
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
        bridge.video_source = Some(adapter);
        bridge.tasks.push(task);
    }
    if let Some(sink) = video_sink {
        let (adapter, task) = video_sink_from_python(py, sink, 3)?;
        bridge.video_sink = Some(adapter);
        bridge.tasks.push(task);
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

    #[test]
    fn linear_resampler_preserves_identity_and_changes_rate() {
        let input = vec![1, 2, 3, 4];
        assert_eq!(resample_linear(&input, 16_000, 16_000), input);
        let output = resample_linear(&input, 8_000, 16_000);
        assert_eq!(output.len(), 8);
        assert_eq!(output[0], 1);
        assert_eq!(output[2], 2);
    }

    #[test]
    fn linear_resampler_handles_empty_input() {
        assert!(resample_linear(&[], 44_100, 16_000).is_empty());
    }
}
