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

	/// Decrypt a poll vote without LID/PN fallback (stateless helper).
	///
	/// Uses `wacore::poll::decrypt_poll_vote_with_secret` directly since this
	/// is a static method with no access to a `Client` for namespace resolution.
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

		wacore::poll::decrypt_poll_vote_with_secret(
			wacore::poll::PollVoteCiphertext { enc_payload, enc_iv },
			message_secret,
			poll_msg_id.as_str(),
			&creator.to_string(),
			&voter.to_string(),
		)
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
	}

	/// Aggregate poll votes without LID/PN fallback (stateless helper).
	///
	/// Uses `wacore::poll` primitives directly since this is a static method
	/// with no access to a `Client` for namespace resolution.
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
		let creator_str = creator.to_string();

		let option_hashes: Vec<([u8; 32], &str)> = poll_options
			.iter()
			.map(|name| (wacore::poll::compute_option_hash(name), name.as_str()))
			.collect();

		let vote_values: Vec<(String, Vec<u8>, Vec<u8>)> = votes
			.iter()
			.map(|(jid, enc_payload, enc_iv)| {
				let voter_jid = jid.bind(py).borrow().as_whatsapp_jid();
				(voter_jid.to_string(), enc_payload.clone(), enc_iv.clone())
			})
			.collect();

		let mut latest_votes: std::collections::HashMap<String, (String, Vec<Vec<u8>>)> =
			std::collections::HashMap::with_capacity(votes.len());

		for (voter_str, enc_payload, enc_iv) in &vote_values {
			match wacore::poll::decrypt_poll_vote_with_secret(
				wacore::poll::PollVoteCiphertext { enc_payload, enc_iv },
				message_secret,
				poll_msg_id.as_str(),
				&creator_str,
				voter_str,
			) {
				Ok(selected_hashes) => {
					if selected_hashes.is_empty() {
						latest_votes.remove(voter_str);
					} else {
						latest_votes.insert(
							voter_str.clone(),
							(voter_str.clone(), selected_hashes),
						);
					}
				}
				Err(_) => {
					// Skip votes that fail to decrypt without a client for fallback
				}
			}
		}

		let mut results: Vec<whatsapp_rust::features::PollOptionResult> = poll_options
			.iter()
			.map(|name| whatsapp_rust::features::PollOptionResult {
				name: name.clone(),
				voters: Vec::new(),
			})
			.collect();

		for (display_jid, selected_hashes) in latest_votes.values() {
			for hash in selected_hashes {
				if let Ok(hash_arr) = <[u8; 32]>::try_from(hash.as_slice()) {
					if let Some(idx) = option_hashes.iter().position(|(h, _)| *h == hash_arr) {
						results[idx].voters.push(display_jid.clone());
					}
				}
			}
		}

		results
			.into_iter()
			.map(|item| Py::new(py, PollOptionResult::from(item)))
			.collect::<PyResult<Vec<_>>>()
	}
}
