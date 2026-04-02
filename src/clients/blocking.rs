use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::iq::blocking::BlocklistEntry;

#[pyclass]
pub struct BlockingClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl BlockingClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running. Call bot.run() or bot.run_blocking() first."))
    }
}

#[pymethods]
impl BlockingClient {
    fn block<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .blocking()
                .block(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn unblock<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .blocking()
                .unblock(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn get_blocklist<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Vec<Py<BlocklistEntry>>>(py, locals, async move {
            let result = client
                .blocking()
                .get_blocklist()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = BlocklistEntry::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn is_blocked<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, bool>(py, locals, async move {
            client
                .blocking()
                .is_blocked(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}