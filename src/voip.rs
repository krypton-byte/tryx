//! Bounded-channel adapters between Python media objects and upstream VoIP traits.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes};
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

static FFMPEG_AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

fn is_ffmpeg_available() -> bool {
    *FFMPEG_AVAILABLE.get_or_init(|| {
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
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
    fn play(&self, py: Python<'_>, path: String) -> PyResult<()> {
        self.stop();
        if !std::path::Path::new(&path).is_file() {
            return Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                "video file not found: {path}"
            )));
        }
        let available = py.detach(is_ffmpeg_available);
        if !available {
            return Err(pyo3::exceptions::PyEnvironmentError::new_err(
                "ffmpeg is not installed or is not available in PATH",
            ));
        }
        let (sender, receiver) = async_channel::bounded(DEFAULT_VIDEO_CAPACITY);
        let (stop_tx, mut stop_rx) = oneshot::channel();
        *self.receiver.lock().map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err("video player lock poisoned")
        })? = Some(receiver);
        let task = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
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
            let mut frames = Vec::new();
            loop {
                let count = tokio::select! {
                    _ = &mut stop_rx => { let _ = child.kill().await; return; },
                    result = stdout.read(&mut buffer) => match result { Ok(0) | Err(_) => break, Ok(n) => n },
                };
                splitter.push(&buffer[..count], &mut frames);
                for frame in frames.drain(..) {
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
        if let Ok(mut stop) = self.stop.lock()
            && let Some(stop) = stop.take()
        {
            let _ = stop.send(());
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
        let task = pyo3_async_runtimes::tokio::get_runtime().spawn(audio_manager(
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
    fn play(&self, py: Python<'_>, path: String, mode: Option<String>) -> PyResult<()> {
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
        self.send_command(py, |reply| AudioCommand::Play(PathBuf::from(path), mode, reply))
    }

    fn stop(&self, py: Python<'_>) {
        let _ = self.send_command(py, AudioCommand::Stop);
    }

    fn pause(&self, py: Python<'_>) {
        let _ = self.send_command(py, AudioCommand::Pause);
    }

    fn resume(&self, py: Python<'_>) {
        let _ = self.send_command(py, AudioCommand::Resume);
    }

    fn enqueue(&self, py: Python<'_>, path: String) -> PyResult<()> {
        self.send_command(py, |reply| AudioCommand::Enqueue(PathBuf::from(path), reply))
    }

    fn skip(&self, py: Python<'_>) -> PyResult<()> {
        self.send_command(py, AudioCommand::Skip)
    }

    fn clear_queue(&self, py: Python<'_>) -> PyResult<()> {
        self.send_command(py, AudioCommand::Clear)
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
    fn send_command<F>(&self, py: Python<'_>, make: F) -> PyResult<()>
    where
        F: FnOnce(oneshot::Sender<Result<(), String>>) -> AudioCommand,
    {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.commands.try_send(make(reply_tx)).map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err("player command queue is closed")
        })?;
        match py.detach(|| reply_rx.blocking_recv()) {
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
        if let Ok(mut task) = self.task.lock()
            && let Some(task) = task.take()
        {
            task.abort();
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
                if interrupted
                    && let Some(active) = current.clone()
                {
                    queue.push_front(active);
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
    let mut samples = Vec::new();
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
        samples.clear();
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
        resample_linear_into(&samples, rate, 16_000, &mut carry);
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
        while paused.load(Ordering::Acquire) {
            if stopped.load(Ordering::Acquire) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        if !stopped.load(Ordering::Acquire) {
            let _ = sender.send_blocking(carry);
        }
    }
    Ok(())
}

/// Resample mono PCM with linear interpolation directly into `output`. Keeping
/// this local avoids a heavyweight resampler state per decoded packet while
/// removing the severe aliasing and stepping artifacts of nearest-neighbour sampling.
fn resample_linear_into(input: &[i16], input_rate: usize, output_rate: usize, output: &mut Vec<i16>) {
    if input.is_empty() {
        return;
    }
    if input_rate == output_rate {
        output.extend_from_slice(input);
        return;
    }
    let output_len = (input.len() as u64 * output_rate as u64).div_ceil(input_rate as u64) as usize;
    output.reserve(output_len);
    let last_idx = input.len() - 1;
    let ratio = input_rate as f64 / output_rate as f64;
    for index in 0..output_len {
        let position = index as f64 * ratio;
        let left = (position.floor() as usize).min(last_idx);
        let right = (left + 1).min(last_idx);
        let fraction = position - left as f64;
        let sample = input[left] as f64 + (input[right] as f64 - input[left] as f64) * fraction;
        output.push(sample.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16);
    }
}

#[cfg(test)]
fn resample_linear(input: &[i16], input_rate: usize, output_rate: usize) -> Vec<i16> {
    let mut output = Vec::new();
    resample_linear_into(input, input_rate, output_rate, &mut output);
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

fn pcm16_bytes_to_samples(bytes: &[u8]) -> Result<Vec<i16>, String> {
    if bytes.len() != AUDIO_FRAME_BYTES {
        return Err(format!(
            "audio frame must be exactly {AUDIO_FRAME_BYTES} bytes (960 PCM16 samples), got {}",
            bytes.len()
        ));
    }
    Ok(bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&v| i16::from_le_bytes(v))
        .collect())
}

fn pcm16_from_python(value: &Bound<'_, PyAny>) -> PyResult<Vec<i16>> {
    if let Ok(py_bytes) = value.cast::<PyBytes>() {
        return pcm16_bytes_to_samples(py_bytes.as_bytes())
            .map_err(pyo3::exceptions::PyValueError::new_err);
    }
    let bytes = value.extract::<Vec<u8>>()?;
    pcm16_bytes_to_samples(&bytes).map_err(pyo3::exceptions::PyValueError::new_err)
}

fn pcm16_to_python(py: Python<'_>, frame: &[i16]) -> PyResult<Py<PyAny>> {
    let bytes = PyBytes::new_with(py, frame.len() * 2, |buf| {
        for (dst, &sample) in buf.as_chunks_mut::<2>().0.iter_mut().zip(frame.iter()) {
            *dst = sample.to_le_bytes();
        }
        Ok(())
    })?;
    Ok(bytes.unbind().into_any())
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
    let task = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
        loop {
            let next = Python::attach(|py| {
                let awaitable = iterator.call_method0(py, "__anext__")?;
                into_future_with_locals(&locals, awaitable.into_bound(py))
            });
            let Ok(next) = next else { break };
            let Ok(value) = next.await else { break };
            let frame = Python::attach(|py| pcm16_from_python(value.bind(py)));
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
    let task = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
        while let Ok(frame) = receiver.recv().await {
            let maybe_future = Python::attach(|py| -> PyResult<_> {
                let bytes = pcm16_to_python(py, &frame)?;
                let result = sink.bind(py).call_method1("write", (bytes,))?;
                if result.hasattr("__await__")? {
                    Ok(Some(into_future_with_locals(&locals, result)?))
                } else {
                    Ok(None)
                }
            });
            match maybe_future {
                Ok(Some(future)) => {
                    if future.await.is_err() {
                        break;
                    }
                }
                Ok(None) => {}
                Err(_) => break,
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
    let task = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
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
                let data_attr = value.bind(py).getattr("data")?;
                let data: Vec<u8> = if let Ok(py_bytes) = data_attr.cast::<PyBytes>() {
                    py_bytes.as_bytes().to_vec()
                } else {
                    data_attr.extract()?
                };
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
    let frame_type = py.import("tryx.media")?.getattr("VideoFrame")?.unbind();
    let (sender, receiver): (Sender<VideoFrame>, Receiver<VideoFrame>) =
        async_channel::bounded(capacity.max(1));
    let task = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
        while let Ok(frame) = receiver.recv().await {
            let maybe_future = Python::attach(|py| -> PyResult<_> {
                let data = PyBytes::new(py, &frame.data);
                let value = frame_type.bind(py).call1((
                    data,
                    py.None(),
                    66_666_i64,
                    frame.keyframe,
                    py.None(),
                    py.None(),
                    frame.orientation,
                ))?;
                let result = sink.bind(py).call_method1("write", (value,))?;
                if result.hasattr("__await__")? {
                    Ok(Some(into_future_with_locals(&locals, result)?))
                } else {
                    Ok(None)
                }
            });
            match maybe_future {
                Ok(Some(future)) => {
                    if future.await.is_err() {
                        break;
                    }
                }
                Ok(None) => {}
                Err(_) => break,
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
            (player.borrow(py).source()?, None)
        } else {
            let (adapter, task) = video_source_from_python(py, source, 3, 90_000 / 15)?;
            (adapter, Some(task))
        };
        bridge.video_source = Some(adapter);
        if let Some(task) = task {
            bridge.tasks.push(task);
        }
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

    #[test]
    fn pcm16_bytes_to_samples_validates_length_and_endianness() {
        let mut raw = vec![0u8; AUDIO_FRAME_BYTES];
        raw[0..2].copy_from_slice(&(-1234i16).to_le_bytes());
        raw[2..4].copy_from_slice(&(32767i16).to_le_bytes());
        let samples = pcm16_bytes_to_samples(&raw).unwrap();
        assert_eq!(samples.len(), AUDIO_FRAME_SAMPLES);
        assert_eq!(samples[0], -1234);
        assert_eq!(samples[1], 32767);
        assert!(pcm16_bytes_to_samples(&[0u8; 4]).is_err());
    }
}
