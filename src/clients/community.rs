use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::iq::community::{
    CommunitySubgroup,
    CreateCommunityOptions,
    CreateCommunityResult,
    GroupMetadata,
    GroupParticipant,
    GroupType,
    LinkSubgroupsResult,
    UnlinkSubgroupsResult,
};

#[pyclass]
pub struct CommunityClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl CommunityClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }
}

#[pymethods]
impl CommunityClient {
    #[staticmethod]
    fn classify_group(metadata: Py<GroupMetadata>) -> GroupType {
        Python::attach(|py| metadata.bind(py).borrow().get_group_type())
    }

    fn create<'py>(
        &self,
        py: Python<'py>,
        options: Py<CreateCommunityOptions>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let options_value = options.bind(py).borrow().to_rust_options();
        if options_value.name.trim().is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Community name cannot be empty",
            ));
        }

        future_into_py_with_locals::<_, Py<CreateCommunityResult>>(py, locals, async move {
            let result = client
                .community()
                .create(options_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = CreateCommunityResult::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn deactivate<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .community()
                .deactivate(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn link_subgroups<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
        subgroup_jids: Vec<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if subgroup_jids.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "subgroup_jids cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let community_jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();
        let subgroup_jid_values = subgroup_jids
            .iter()
            .map(|jid| jid.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals::<_, Py<LinkSubgroupsResult>>(py, locals, async move {
            let result = client
                .community()
                .link_subgroups(&community_jid_value, subgroup_jid_values.as_slice())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = LinkSubgroupsResult::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn unlink_subgroups<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
        subgroup_jids: Vec<Py<JID>>,
        remove_orphan_members: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        if subgroup_jids.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "subgroup_jids cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let community_jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();
        let subgroup_jid_values = subgroup_jids
            .iter()
            .map(|jid| jid.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();

        future_into_py_with_locals::<_, Py<UnlinkSubgroupsResult>>(py, locals, async move {
            let result = client
                .community()
                .unlink_subgroups(
                    &community_jid_value,
                    subgroup_jid_values.as_slice(),
                    remove_orphan_members,
                )
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = UnlinkSubgroupsResult::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn get_subgroups<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let community_jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Vec<Py<CommunitySubgroup>>>(py, locals, async move {
            let result = client
                .community()
                .get_subgroups(&community_jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = CommunitySubgroup::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn get_subgroup_participant_counts<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let community_jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Vec<(Py<JID>, u32)>>(py, locals, async move {
            let result = client
                .community()
                .get_subgroup_participant_counts(&community_jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|(jid, count)| Ok((Py::new(py, JID::from(jid))?, count)))
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn query_linked_group<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
        subgroup_jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let community_jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();
        let subgroup_jid_value = subgroup_jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Py<GroupMetadata>>(py, locals, async move {
            let result = client
                .community()
                .query_linked_group(&community_jid_value, &subgroup_jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = GroupMetadata::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn join_subgroup<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
        subgroup_jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let community_jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();
        let subgroup_jid_value = subgroup_jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Py<GroupMetadata>>(py, locals, async move {
            let result = client
                .community()
                .join_subgroup(&community_jid_value, &subgroup_jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_result = GroupMetadata::from_inner(py, result)?;
                Py::new(py, py_result)
            })
        })
    }

    fn get_linked_groups_participants<'py>(
        &self,
        py: Python<'py>,
        community_jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let community_jid_value = community_jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Vec<Py<GroupParticipant>>>(py, locals, async move {
            let result = client
                .community()
                .get_linked_groups_participants(&community_jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = GroupParticipant::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }
}
