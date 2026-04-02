use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::iq::presence::PresenceStatus;

#[pyclass]
pub struct PresenceClient {
	pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl PresenceClient {
	fn get_client(&self) -> PyResult<Arc<Client>> {
		self.client_rx
			.borrow()
			.clone()
			.ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
	}
}

#[pymethods]
impl PresenceClient {
	fn set<'py>(
		&self,
		py: Python<'py>,
		status: PresenceStatus,
	) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let status_value: whatsapp_rust::PresenceStatus = status.into();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.presence()
				.set(status_value)
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}

	fn set_available<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.presence()
				.set_available()
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}

	fn set_unavailable<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.presence()
				.set_unavailable()
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}

	fn subscribe<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.presence()
				.subscribe(&jid_value)
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}

	fn unsubscribe<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.presence()
				.unsubscribe(&jid_value)
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}
}
