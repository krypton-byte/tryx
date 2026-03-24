use pyo3::{Py, PyResult, Python, pyclass, pymethods};

use crate::types::JID;
use crate::wacore::iq::polls::PollOptionResult;

#[pyclass]
pub struct PollsHelpers;

#[pymethods]
impl PollsHelpers {
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
        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
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
        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        results
            .into_iter()
            .map(|item| Py::new(py, PollOptionResult::from(item)))
            .collect::<PyResult<Vec<_>>>()
    }
}
