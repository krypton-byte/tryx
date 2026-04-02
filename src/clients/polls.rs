use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::iq::polls::PollOptionResult;

#[pyclass]
pub struct PollsClient {
	pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl PollsClient {
	fn get_client(&self) -> PyResult<Arc<Client>> {
		self.client_rx
			.borrow()
			.clone()
			.ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
	}
}

#[pymethods]
impl PollsClient {
	fn create<'py>(
		&self,
		py: Python<'py>,
		to: Py<JID>,
		name: String,
		options: Vec<String>,
		selectable_count: u32,
	) -> PyResult<Bound<'py, PyAny>> {
		if options.is_empty() {
			return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
				"options cannot be empty",
			));
		}

		let client = self.get_client()?;
		let to_value = to.bind(py).borrow().as_whatsapp_jid();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals::<_, (String, Vec<u8>)>(py, locals, async move {
			client
				.polls()
				.create(&to_value, name.as_str(), options.as_slice(), selectable_count)
				.await
				.map(|(result, secret)| (result.message_id, secret))
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
		})
	}

	fn vote<'py>(
		&self,
		py: Python<'py>,
		chat_jid: Py<JID>,
		poll_msg_id: String,
		poll_creator_jid: Py<JID>,
		message_secret: &[u8],
		option_names: Vec<String>,
	) -> PyResult<Bound<'py, PyAny>> {
		if option_names.is_empty() {
			return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
				"option_names cannot be empty",
			));
		}

		let client = self.get_client()?;
		let chat_jid_value = chat_jid.bind(py).borrow().as_whatsapp_jid();
		let creator_jid_value = poll_creator_jid.bind(py).borrow().as_whatsapp_jid();
		let message_secret_value = message_secret.to_vec();
		let locals = get_current_locals(py)?;

		future_into_py_with_locals::<_, String>(py, locals, async move {
			client
				.polls()
				.vote(
					&chat_jid_value,
					poll_msg_id.as_str(),
					&creator_jid_value,
					message_secret_value.as_slice(),
					option_names.as_slice(),
				)
				.await
				.map(|result| result.message_id)
				.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
		})
	}

	#[staticmethod]
	fn decrypt_vote(
		py: Python<'_>,
		enc_payload: &[u8],
		enc_iv: &[u8],
		message_secret: &[u8],
		poll_msg_id: String,
		poll_creator_jid: Py<JID>,
		voter_jid: Py<JID>,
	) -> PyResult<Vec<Vec<u8>>> {
		let creator = poll_creator_jid.bind(py).borrow().as_whatsapp_jid();
		let voter = voter_jid.bind(py).borrow().as_whatsapp_jid();

		whatsapp_rust::features::Polls::decrypt_vote(
			enc_payload,
			enc_iv,
			message_secret,
			poll_msg_id.as_str(),
			&creator,
			&voter,
		)
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
	}

	#[staticmethod]
	fn aggregate_votes(
		py: Python<'_>,
		poll_options: Vec<String>,
		votes: Vec<(Py<JID>, Vec<u8>, Vec<u8>)>,
		message_secret: &[u8],
		poll_msg_id: String,
		poll_creator_jid: Py<JID>,
	) -> PyResult<Vec<Py<PollOptionResult>>> {
		let creator = poll_creator_jid.bind(py).borrow().as_whatsapp_jid();

		let vote_values = votes
			.iter()
			.map(|(jid, enc_payload, enc_iv)| {
				(
					jid.bind(py).borrow().as_whatsapp_jid(),
					enc_payload.as_slice(),
					enc_iv.as_slice(),
				)
			})
			.collect::<Vec<_>>();

		let refs = vote_values
			.iter()
			.map(|(jid, enc_payload, enc_iv)| (jid, *enc_payload, *enc_iv))
			.collect::<Vec<_>>();

		let results = whatsapp_rust::features::Polls::aggregate_votes(
			poll_options.as_slice(),
			refs.as_slice(),
			message_secret,
			poll_msg_id.as_str(),
			&creator,
		)
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

		results
			.into_iter()
			.map(|item| Py::new(py, PollOptionResult::from(item)))
			.collect::<PyResult<Vec<_>>>()
	}
}
