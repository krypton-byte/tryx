use std::sync::Arc;
use std::time::Duration;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3::types::{PyDict, PyDictMethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::node::Node;

#[pyclass]
pub struct AdvancedClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl AdvancedClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
    }

    fn stats_dict(py: Python<'_>, stats: whatsapp_rust::StatsSnapshot) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("bytes_sent", stats.bytes_sent)?;
        dict.set_item("bytes_received", stats.bytes_received)?;
        dict.set_item("frames_sent", stats.frames_sent)?;
        dict.set_item("frames_received", stats.frames_received)?;
        dict.set_item("messages_sent", stats.messages_sent)?;
        dict.set_item("messages_received", stats.messages_received)?;
        dict.set_item("events_dropped", stats.events_dropped)?;
        dict.set_item("reconnects", stats.reconnects)?;
        dict.set_item("reconnect_errors", stats.reconnect_errors)?;
        dict.set_item("resends_throttled", stats.resends_throttled)?;
        dict.set_item("last_data_received_ms", stats.last_data_received_ms)?;
        Ok(dict.unbind())
    }
}

#[pymethods]
impl AdvancedClient {
    fn is_logged_in(&self) -> PyResult<bool> {
        Ok(self.get_client()?.is_logged_in())
    }

    fn get_push_name(&self) -> PyResult<String> {
        Ok(self.get_client()?.push_name())
    }

    fn get_pn<'py>(&self, py: Python<'py>) -> PyResult<Option<Py<JID>>> {
        self.get_client()?
            .pn()
            .map(|jid| Py::new(py, JID::from(jid)))
            .transpose()
    }

    fn get_lid<'py>(&self, py: Python<'py>) -> PyResult<Option<Py<JID>>> {
        self.get_client()?
            .lid()
            .map(|jid| Py::new(py, JID::from(jid)))
            .transpose()
    }

    fn stats<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let stats = self.get_client()?.stats();
        Ok(Self::stats_dict(py, stats)?.into_bound(py).into_any())
    }

    fn memory_report_text<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, String>(py, locals, async move {
            Ok(client.memory_report().await.to_string())
        })
    }

    fn resource_report_text<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, String>(py, locals, async move {
            Ok(client.resource_report().await.to_string())
        })
    }

    fn wait_for_socket<'py>(&self, py: Python<'py>, timeout_seconds: f64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .wait_for_socket(Duration::from_secs_f64(timeout_seconds))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyTimeoutError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn wait_for_connected<'py>(&self, py: Python<'py>, timeout_seconds: f64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .wait_for_connected(Duration::from_secs_f64(timeout_seconds))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyTimeoutError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn wait_for_startup_sync<'py>(&self, py: Python<'py>, timeout_seconds: f64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .wait_for_startup_sync(Duration::from_secs_f64(timeout_seconds))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyTimeoutError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn flush_pending_signal_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .flush_pending_signal_state()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn send_raw_bytes<'py>(&self, py: Python<'py>, plaintext: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .send_raw_bytes(plaintext)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn send_node<'py>(&self, py: Python<'py>, node: Py<Node>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let node_value = node.bind(py).borrow().to_node_builder(py).build();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .send_node(node_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn set_force_active_delivery_receipts(&self, active: bool) -> PyResult<()> {
        self.get_client()?.set_force_active_delivery_receipts(active);
        Ok(())
    }
}
