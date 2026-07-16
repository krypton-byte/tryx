use std::sync::Arc;

use pyo3::{Bound, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;

#[pyclass]
pub struct LabelsClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl LabelsClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
    }
}

#[pymethods]
impl LabelsClient {
    fn create_label<'py>(&self, py: Python<'py>, label_id: String, name: String, color: i32) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .labels()
                .create_label(label_id.as_str(), name.as_str(), color)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn delete_label<'py>(&self, py: Python<'py>, label_id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .labels()
                .delete_label(label_id.as_str())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn add_chat_label<'py>(&self, py: Python<'py>, jid: pyo3::Py<JID>, label_id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .labels()
                .add_chat_label(label_id.as_str(), &jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn remove_chat_label<'py>(&self, py: Python<'py>, jid: pyo3::Py<JID>, label_id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .labels()
                .remove_chat_label(label_id.as_str(), &jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }
}
