use std::sync::Arc;

use buffa::Message;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use waproto::whatsapp::Message as WhatsappMessage;
use whatsapp_rust::Client;

use crate::events::types::EvMessage;
use whatsapp_rust::waproto::whatsapp as wa;

#[pyclass]
pub struct CommentsClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl CommentsClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
    }

    fn parent_key(parent: &EvMessage) -> wa::MessageKey {
        let info = parent.inner_message_info.as_ref();
        wa::MessageKey {
            remote_jid: Some(info.source.chat.to_string()),
            from_me: Some(info.source.is_from_me),
            id: Some(info.id.to_string()),
            participant: if info.source.is_group {
                Some(info.source.sender.to_string())
            } else {
                None
            },
            ..Default::default()
        }
    }
}

#[pymethods]
impl CommentsClient {
    fn send_text<'py>(&self, py: Python<'py>, parent: Py<EvMessage>, text: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let parent_ref = parent.bind(py).borrow();
        let chat = parent_ref.inner_message_info.source.chat.clone();
        let parent_key = Self::parent_key(&parent_ref);
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .comments()
                .send_text(chat, parent_key, text.as_str())
                .await
                .map(|result| result.message_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn send_message<'py>(&self, py: Python<'py>, parent: Py<EvMessage>, message: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let parent_ref = parent.bind(py).borrow();
        let chat = parent_ref.inner_message_info.source.chat.clone();
        let parent_key = Self::parent_key(&parent_ref);
        let serialized: Vec<u8> = message.call_method0(py, "SerializeToString")?.extract(py)?;
        let message_value = WhatsappMessage::decode(&mut serialized.as_slice()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to decode WhatsAppMessage proto: {}", e),
            )
        })?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .comments()
                .send_message(chat, parent_key, message_value)
                .await
                .map(|result| result.message_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}
