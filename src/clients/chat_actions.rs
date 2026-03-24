use std::sync::Arc;

use prost::Message;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use waproto::whatsapp::sync_action_value::SyncActionMessageRange;
use whatsapp_rust::Client;

use crate::types::JID;

#[pyclass]
pub struct ChatActionsClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl ChatActionsClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }
}

#[pymethods]
impl ChatActionsClient {
    fn archive_chat<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        message_range: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        let parsed_range = if let Some(range) = message_range {
            let serialized: Vec<u8> = range.call_method0(py, "SerializeToString")?.extract(py)?;
            Some(SyncActionMessageRange::decode(serialized.as_slice()).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to decode SyncActionMessageRange: {}", e),
                )
            })?)
        } else {
            None
        };

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .archive_chat(&jid_value, parsed_range)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }
}