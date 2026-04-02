use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::iq::chatstate::ChatStateType;

#[pyclass]
pub struct ChatstateClient {
	pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl ChatstateClient {
	fn get_client(&self) -> PyResult<Arc<Client>> {
		self.client_rx
			.borrow()
			.clone()
			.ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
	}
}

#[pymethods]
impl ChatstateClient {
	fn send<'py>(
		&self,
		py: Python<'py>,
		to: Py<JID>,
		state: ChatStateType,
	) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let to_value = to.bind(py).borrow().as_whatsapp_jid();
		let state_value: whatsapp_rust::ChatStateType = state.into();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.chatstate()
				.send(&to_value, state_value)
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}

	fn send_composing<'py>(&self, py: Python<'py>, to: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let to_value = to.bind(py).borrow().as_whatsapp_jid();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.chatstate()
				.send_composing(&to_value)
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}

	fn send_recording<'py>(&self, py: Python<'py>, to: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let to_value = to.bind(py).borrow().as_whatsapp_jid();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.chatstate()
				.send_recording(&to_value)
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}

	fn send_paused<'py>(&self, py: Python<'py>, to: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
		let client = self.get_client()?;
		let to_value = to.bind(py).borrow().as_whatsapp_jid();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals(py, locals, async move {
			client
				.chatstate()
				.send_paused(&to_value)
				.await
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
			Ok(())
		})
	}
}
