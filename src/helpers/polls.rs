use pyo3::{Py, PyResult, Python, pyclass, pymethods};

use crate::types::JID;
use crate::wacore::iq::polls::PollOptionResult;

#[pyclass]
pub struct PollsHelpers;

#[pymethods]
impl PollsHelpers {
    /// Decrypt a poll vote using `wacore::poll` primitives directly.
    ///
    /// No LID/PN fallback is performed since this is a stateless helper
    /// without access to a `Client` instance.
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
        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Aggregate poll votes using `wacore::poll` primitives directly.
    ///
    /// No LID/PN fallback is performed since this is a stateless helper
    /// without access to a `Client` instance.
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
