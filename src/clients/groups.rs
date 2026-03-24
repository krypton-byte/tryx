use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3::types::{PyDict, PyDictMethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::iq::community::GroupMetadata;
use crate::wacore::iq::groups::{
    CreateGroupOptions,
    CreateGroupResult,
    GroupInfo,
    JoinGroupResult,
    MemberAddMode,
    MembershipApprovalMode,
    MembershipRequest,
    ParticipantChangeResponse,
};

#[pyclass]
pub struct GroupsClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl GroupsClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }
}

#[pymethods]
impl GroupsClient {
    fn query_info<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Py<GroupInfo>>(py, locals, async move {
            let result = client
                .groups()
                .query_info(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = GroupInfo::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn get_participating<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Py<PyDict>>(py, locals, async move {
            let result = client
                .groups()
                .get_participating()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let dict = PyDict::new(py);
                for (jid, meta) in result {
                    let py_meta = GroupMetadata::from_inner(py, meta)?;
                    dict.set_item(jid, Py::new(py, py_meta)?)?;
                }
                Ok(dict.unbind())
            })
        })
    }

    fn get_metadata<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Py<GroupMetadata>>(py, locals, async move {
            let result = client
                .groups()
                .get_metadata(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = GroupMetadata::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn create_group<'py>(
        &self,
        py: Python<'py>,
        options: Py<CreateGroupOptions>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let options_value = options.bind(py).borrow().to_rust_options(py);

        future_into_py_with_locals::<_, Py<CreateGroupResult>>(py, locals, async move {
            let result = client
                .groups()
                .create_group(options_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = CreateGroupResult::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn set_subject<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        subject: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let subject_value = whatsapp_rust::GroupSubject::new(subject.as_str()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
        })?;
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .set_subject(&jid_value, subject_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    #[pyo3(signature = (jid, description=None, prev=None))]
    fn set_description<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        description: Option<String>,
        prev: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let description_value = match description {
            Some(value) => Some(whatsapp_rust::GroupDescription::new(value.as_str()).map_err(
                |e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()),
            )?),
            None => None,
        };
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .set_description(&jid_value, description_value, prev)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn leave<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .leave(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn add_participants<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        participants: Vec<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if participants.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "participants cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let participant_values = participants
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals::<_, Vec<Py<ParticipantChangeResponse>>>(py, locals, async move {
            let result = client
                .groups()
                .add_participants(&jid_value, participant_values.as_slice())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = ParticipantChangeResponse::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn remove_participants<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        participants: Vec<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if participants.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "participants cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let participant_values = participants
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals::<_, Vec<Py<ParticipantChangeResponse>>>(py, locals, async move {
            let result = client
                .groups()
                .remove_participants(&jid_value, participant_values.as_slice())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = ParticipantChangeResponse::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn promote_participants<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        participants: Vec<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if participants.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "participants cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let participant_values = participants
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .promote_participants(&jid_value, participant_values.as_slice())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn demote_participants<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        participants: Vec<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if participants.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "participants cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let participant_values = participants
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .demote_participants(&jid_value, participant_values.as_slice())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn get_invite_link<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        reset: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .groups()
                .get_invite_link(&jid_value, reset)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn set_locked<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        locked: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .set_locked(&jid_value, locked)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn set_announce<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        announce: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .set_announce(&jid_value, announce)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn set_ephemeral<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        expiration: u32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .set_ephemeral(&jid_value, expiration)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn set_membership_approval<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        mode: MembershipApprovalMode,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let mode_value: whatsapp_rust::MembershipApprovalMode = mode.into();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .set_membership_approval(&jid_value, mode_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn join_with_invite_code<'py>(
        &self,
        py: Python<'py>,
        code: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Py<JoinGroupResult>>(py, locals, async move {
            let result = client
                .groups()
                .join_with_invite_code(code.as_str())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = JoinGroupResult::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn join_with_invite_v4<'py>(
        &self,
        py: Python<'py>,
        group_jid: Py<JID>,
        code: String,
        expiration: i64,
        admin_jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let group_jid_value = group_jid.bind(py).borrow().as_whatsapp_jid();
        let admin_jid_value = admin_jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Py<JoinGroupResult>>(py, locals, async move {
            let result = client
                .groups()
                .join_with_invite_v4(&group_jid_value, code.as_str(), expiration, &admin_jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = JoinGroupResult::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn get_invite_info<'py>(
        &self,
        py: Python<'py>,
        code: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Py<GroupMetadata>>(py, locals, async move {
            let result = client
                .groups()
                .get_invite_info(code.as_str())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = GroupMetadata::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn get_membership_requests<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Vec<Py<MembershipRequest>>>(py, locals, async move {
            let result = client
                .groups()
                .get_membership_requests(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = MembershipRequest::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn approve_membership_requests<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        participants: Vec<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if participants.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "participants cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let participant_values = participants
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals::<_, Vec<Py<ParticipantChangeResponse>>>(py, locals, async move {
            let result = client
                .groups()
                .approve_membership_requests(&jid_value, participant_values.as_slice())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = ParticipantChangeResponse::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn reject_membership_requests<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        participants: Vec<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if participants.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "participants cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let participant_values = participants
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals::<_, Vec<Py<ParticipantChangeResponse>>>(py, locals, async move {
            let result = client
                .groups()
                .reject_membership_requests(&jid_value, participant_values.as_slice())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = ParticipantChangeResponse::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn set_member_add_mode<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        mode: MemberAddMode,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let mode_value: whatsapp_rust::MemberAddMode = mode.into();

        future_into_py_with_locals(py, locals, async move {
            client
                .groups()
                .set_member_add_mode(&jid_value, mode_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }
}