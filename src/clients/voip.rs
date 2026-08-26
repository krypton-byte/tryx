use std::sync::Arc;

use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::voip::bridge_from_python;
use whatsapp_rust::types::call::IncomingCall;
use whatsapp_rust::wacore::types::group_call::CallLinkMedia;

#[pyclass]
pub struct VoipClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl VoipClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx.borrow().clone().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err(
                "Client is not running. Call Tryx.run() first.",
            )
        })
    }
}

#[pyclass]
pub struct CallHandle {
    inner: Arc<whatsapp_rust::voip::CallHandle>,
    bridge_tasks: std::sync::Mutex<Vec<tokio::task::JoinHandle<()>>>,
}

#[pyclass]
pub struct IncomingCallEvent {
    pub(crate) client: Arc<Client>,
    pub(crate) incoming: std::sync::Mutex<Option<IncomingCall>>,
}

#[pymethods]
impl IncomingCallEvent {
    #[getter]
    fn call_id(&self) -> PyResult<String> {
        let incoming = self.incoming.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("incoming call lock poisoned"))?;
        incoming.as_ref().map(|v| v.action.call_id().to_string()).ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("incoming call already consumed"))
    }

    #[getter]
    fn peer(&self, py: Python<'_>) -> PyResult<Py<JID>> {
        let incoming = self.incoming.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("incoming call lock poisoned"))?;
        let value = incoming.as_ref().ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("incoming call already consumed"))?;
        Py::new(py, JID::from(value.from.clone()))
    }

    #[getter]
    fn is_video(&self) -> PyResult<bool> {
        let incoming = self.incoming.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("incoming call lock poisoned"))?;
        let value = incoming.as_ref().ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("incoming call already consumed"))?;
        Ok(matches!(value.action, whatsapp_rust::types::call::CallAction::Offer { is_video: true, .. }))
    }

    fn reject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let incoming = self.incoming.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("incoming call lock poisoned"))?.take().ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("incoming call already consumed"))?;
        let client = self.client.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client.voip().reject(&incoming).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    fn accept<'py>(&self, py: Python<'py>, audio_source: Py<PyAny>, audio_sink: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let incoming = self.incoming.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("incoming call lock poisoned"))?.take().ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("incoming call already consumed"))?;
        let bridge = bridge_from_python(py, Some(audio_source), Some(audio_sink), None, None)?;
        let source = bridge.audio_source.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio source adapter is missing"))?;
        let sink = bridge.audio_sink.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio sink adapter is missing"))?;
        let tasks = bridge.tasks;
        let client = self.client.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<CallHandle>>(py, locals, async move {
            let result = client.voip().accept(&incoming).audio(source, sink).start().await;
            let handle = match result {
                Ok(handle) => handle,
                Err(error) => { for task in tasks { task.abort(); } return Err(pyo3::exceptions::PyRuntimeError::new_err(error.to_string())); }
            };
            Python::attach(|py| Py::new(py, CallHandle { inner: Arc::new(handle), bridge_tasks: std::sync::Mutex::new(tasks) }))
        })
    }
}

impl Drop for CallHandle {
    fn drop(&mut self) {
        if let Ok(mut tasks) = self.bridge_tasks.lock() {
            for task in tasks.drain(..) {
                task.abort();
            }
        }
    }
}

#[pymethods]
impl CallHandle {
    #[getter]
    fn call_id(&self) -> String {
        self.inner.call_id().to_string()
    }

    #[getter]
    fn peer(&self, py: Python<'_>) -> PyResult<Py<JID>> {
        Py::new(py, JID::from(self.inner.peer_jid()))
    }

    fn is_muted(&self) -> bool {
        self.inner.is_muted()
    }

    fn set_muted<'py>(&self, py: Python<'py>, muted: bool) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            let _ = call.set_muted(muted).await;
            Ok(())
        })
    }

    fn hangup<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            call.hangup_local().await;
            Ok(())
        })
    }

    fn wait_ended<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            call.wait_ended().await;
            Ok(())
        })
    }

    fn start_video<'py>(&self, py: Python<'py>, video_source: Py<PyAny>, video_sink: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let bridge = crate::voip::bridge_from_python(py, None, None, Some(video_source), Some(video_sink))?;
        let source = bridge.video_source.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("video source adapter is missing"))?;
        let sink = bridge.video_sink.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("video sink adapter is missing"))?;
        let tasks = bridge.tasks;
        self.bridge_tasks.lock().map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("call task lock poisoned"))?.extend(tasks);
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            call.start_video(source, sink).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Ok(())
        })
    }

    fn stop_video<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            call.stop_video().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Ok(())
        })
    }

    fn invite_participant<'py>(&self, py: Python<'py>, target: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let target = target.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move { call.invite_participant(&target).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string())) })
    }

    fn ring_participant<'py>(&self, py: Python<'py>, target: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let target = target.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move { call.ring_participant(&target).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string())) })
    }

    fn start_screen_share<'py>(&self, py: Python<'py>, screen_share_id: Option<u32>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move { call.start_screen_share(screen_share_id).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string())) })
    }

    fn stop_screen_share<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move { call.stop_screen_share().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string())) })
    }

    fn set_approval_required<'py>(&self, py: Python<'py>, enabled: bool) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move { call.set_approval_required(enabled).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string())) })
    }

    fn admit_waiting_user<'py>(&self, py: Python<'py>, target: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone(); let target = target.bind(py).borrow().as_whatsapp_jid(); let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move { call.admit_waiting_user(&target).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string())) })
    }

    fn deny_waiting_user<'py>(&self, py: Python<'py>, target: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let call = self.inner.clone(); let target = target.bind(py).borrow().as_whatsapp_jid(); let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move { call.deny_waiting_user(&target).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string())) })
    }
}

#[pymethods]
impl VoipClient {
    /// Start an outgoing 1:1 PCM audio call. `audio_source` must expose
    /// `frames() -> async iterator[bytes]`; `audio_sink` must expose
    /// `write(bytes) -> awaitable`. Each audio frame is 960 little-endian
    /// signed-16-bit mono samples at 16 kHz.
    fn call<'py>(
        &self,
        py: Python<'py>,
        peer: Py<JID>,
        audio_source: Py<PyAny>,
        audio_sink: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let peer_value = peer.bind(py).borrow().as_whatsapp_jid();
        let bridge = bridge_from_python(py, Some(audio_source), Some(audio_sink), None, None)?;
        let source = bridge.audio_source.ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("audio source adapter is missing")
        })?;
        let sink = bridge.audio_sink.ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("audio sink adapter is missing")
        })?;
        let mut tasks = bridge.tasks;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<CallHandle>>(py, locals, async move {
            let result = client
                .voip()
                .call(&peer_value)
                .audio(source, sink)
                .start()
                .await;
            let handle = match result {
                Ok(handle) => handle,
                Err(error) => {
                    for task in tasks.drain(..) {
                        task.abort();
                    }
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(error.to_string()));
                }
            };
            Python::attach(|py| {
                Py::new(
                    py,
                    CallHandle {
                        inner: Arc::new(handle),
                        bridge_tasks: std::sync::Mutex::new(tasks),
                    },
                )
            })
        })
    }

    /// Start an outgoing 1:1 call with Rust-side H.264 video source/sink.
    fn video_call<'py>(
        &self, py: Python<'py>, peer: Py<JID>, audio_source: Py<PyAny>,
        audio_sink: Py<PyAny>, video_source: Py<PyAny>, video_sink: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let peer_value = peer.bind(py).borrow().as_whatsapp_jid();
        let bridge = bridge_from_python(py, Some(audio_source), Some(audio_sink), Some(video_source), Some(video_sink))?;
        let source = bridge.audio_source.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio source adapter is missing"))?;
        let sink = bridge.audio_sink.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio sink adapter is missing"))?;
        let video_source = bridge.video_source.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("video source adapter is missing"))?;
        let video_sink = bridge.video_sink.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("video sink adapter is missing"))?;
        let mut tasks = bridge.tasks;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<CallHandle>>(py, locals, async move {
            let result = client.voip().call(&peer_value).audio(source, sink).video(video_source, video_sink).start().await;
            let handle = match result {
                Ok(handle) => handle,
                Err(error) => { for task in tasks.drain(..) { task.abort(); } return Err(pyo3::exceptions::PyRuntimeError::new_err(error.to_string())); }
            };
            Python::attach(|py| Py::new(py, CallHandle { inner: Arc::new(handle), bridge_tasks: std::sync::Mutex::new(tasks) }))
        })
    }

    fn group_call<'py>(&self, py: Python<'py>, peers: Vec<Py<JID>>, audio_source: Py<PyAny>, audio_sink: Py<PyAny>, video_source: Option<Py<PyAny>>, video_sink: Option<Py<PyAny>>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let targets: Vec<_> = peers.iter().map(|p| p.bind(py).borrow().as_whatsapp_jid()).collect();
        let bridge = bridge_from_python(py, Some(audio_source), Some(audio_sink), video_source, video_sink)?;
        let source = bridge.audio_source.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio source adapter is missing"))?;
        let sink = bridge.audio_sink.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio sink adapter is missing"))?;
        let video = bridge.video_source.zip(bridge.video_sink); let mut tasks = bridge.tasks; let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<CallHandle>>(py, locals, async move {
            let voip = client.voip();
            let mut builder = voip.group_call(&targets).audio(source, sink);
            if let Some((vs, vk)) = video { builder = builder.video(vs, vk); }
            let result = builder.start().await;
            let handle = match result { Ok(v) => v, Err(e) => { for task in tasks.drain(..) { task.abort(); } return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())); } };
            Python::attach(|py| Py::new(py, CallHandle { inner: Arc::new(handle), bridge_tasks: std::sync::Mutex::new(tasks) }))
        })
    }

    fn join_call_link<'py>(&self, py: Python<'py>, token_or_url: String, media: String, audio_source: Py<PyAny>, audio_sink: Py<PyAny>, video_source: Option<Py<PyAny>>, video_sink: Option<Py<PyAny>>) -> PyResult<Bound<'py, PyAny>> {
        let media = match media.as_str() { "audio" => CallLinkMedia::Audio, "video" => CallLinkMedia::Video, _ => return Err(pyo3::exceptions::PyValueError::new_err("media must be audio or video")) };
        let client = self.get_client()?; let bridge = bridge_from_python(py, Some(audio_source), Some(audio_sink), video_source, video_sink)?;
        let source = bridge.audio_source.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio source adapter is missing"))?; let sink = bridge.audio_sink.ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("audio sink adapter is missing"))?; let video = bridge.video_source.zip(bridge.video_sink); let mut tasks = bridge.tasks; let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<CallHandle>>(py, locals, async move {
            let voip = client.voip();
            let mut builder = voip.call_link(&token_or_url, media).audio(source, sink); if let Some((vs, vk)) = video { builder = builder.video(vs, vk); } let result = builder.start().await;
            let handle = match result { Ok(v) => v, Err(e) => { for task in tasks.drain(..) { task.abort(); } return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())); } }; Python::attach(|py| Py::new(py, CallHandle { inner: Arc::new(handle), bridge_tasks: std::sync::Mutex::new(tasks) }))
        })
    }
}
