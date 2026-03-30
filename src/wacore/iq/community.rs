use pyo3::prelude::*;
use wacore::types::message::AddressingMode;
use whatsapp_rust::{
    CommunitySubgroup as WaCommunitySubgroup,
    CreateCommunityOptions as WaCreateCommunityOptions,
    CreateCommunityResult as WaCreateCommunityResult,
    GroupMetadata as WaGroupMetadata,
    GroupParticipant as WaGroupParticipant,
    GroupType as WaGroupType,
    LinkSubgroupsResult as WaLinkSubgroupsResult,
    UnlinkSubgroupsResult as WaUnlinkSubgroupsResult,
};

use crate::types::JID;

#[pyclass(eq, eq_int, skip_from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupType {
    Default = 0,
    Community = 1,
    LinkedSubgroup = 2,
    LinkedAnnouncementGroup = 3,
    LinkedGeneralGroup = 4,
}

impl From<WaGroupType> for GroupType {
    fn from(value: WaGroupType) -> Self {
        match value {
            WaGroupType::Default => Self::Default,
            WaGroupType::Community => Self::Community,
            WaGroupType::LinkedSubgroup => Self::LinkedSubgroup,
            WaGroupType::LinkedAnnouncementGroup => Self::LinkedAnnouncementGroup,
            WaGroupType::LinkedGeneralGroup => Self::LinkedGeneralGroup,
            _ => Self::Default,
        }
    }
}

#[pyclass]
pub struct CreateCommunityOptions {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub description: Option<String>,
    #[pyo3(get, set)]
    pub closed: bool,
    #[pyo3(get, set)]
    pub allow_non_admin_sub_group_creation: bool,
    #[pyo3(get, set)]
    pub create_general_chat: bool,
}

impl CreateCommunityOptions {
    pub fn to_rust_options(&self) -> WaCreateCommunityOptions {
        WaCreateCommunityOptions {
            name: self.name.clone(),
            description: self.description.clone(),
            closed: self.closed,
            allow_non_admin_sub_group_creation: self.allow_non_admin_sub_group_creation,
            create_general_chat: self.create_general_chat,
        }
    }
}

#[pymethods]
impl CreateCommunityOptions {
    #[new]
    #[pyo3(signature = (
        name,
        description = None,
        closed = false,
        allow_non_admin_sub_group_creation = false,
        create_general_chat = true
    ))]
    fn new(
        name: String,
        description: Option<String>,
        closed: bool,
        allow_non_admin_sub_group_creation: bool,
        create_general_chat: bool,
    ) -> Self {
        Self {
            name,
            description,
            closed,
            allow_non_admin_sub_group_creation,
            create_general_chat,
        }
    }
}

#[pyclass]
pub struct CreateCommunityResult {
    #[pyo3(get)]
    pub gid: Py<JID>,
}

impl CreateCommunityResult {
    pub fn from_inner(py: Python<'_>, value: WaCreateCommunityResult) -> PyResult<Self> {
        Ok(Self {
            gid: Py::new(py, JID::from(value.gid))?,
        })
    }
}

#[pyclass]
pub struct CommunitySubgroup {
    #[pyo3(get)]
    pub id: Py<JID>,
    #[pyo3(get)]
    pub subject: String,
    #[pyo3(get)]
    pub participant_count: Option<u32>,
    #[pyo3(get)]
    pub is_default_sub_group: bool,
    #[pyo3(get)]
    pub is_general_chat: bool,
}

impl CommunitySubgroup {
    pub fn from_inner(py: Python<'_>, value: WaCommunitySubgroup) -> PyResult<Self> {
        Ok(Self {
            id: Py::new(py, JID::from(value.id))?,
            subject: value.subject,
            participant_count: value.participant_count,
            is_default_sub_group: value.is_default_sub_group,
            is_general_chat: value.is_general_chat,
        })
    }
}

#[pyclass]
pub struct LinkSubgroupsResult {
    #[pyo3(get)]
    pub linked_jids: Vec<Py<JID>>,
    #[pyo3(get)]
    pub failed_groups: Vec<(Py<JID>, u32)>,
}

impl LinkSubgroupsResult {
    pub fn from_inner(py: Python<'_>, value: WaLinkSubgroupsResult) -> PyResult<Self> {
        let linked_jids = value
            .linked_jids
            .into_iter()
            .map(|jid| Py::new(py, JID::from(jid)))
            .collect::<PyResult<Vec<_>>>()?;
        let failed_groups = value
            .failed_groups
            .into_iter()
            .map(|(jid, code)| Ok((Py::new(py, JID::from(jid))?, code)))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self {
            linked_jids,
            failed_groups,
        })
    }
}

#[pyclass]
pub struct UnlinkSubgroupsResult {
    #[pyo3(get)]
    pub unlinked_jids: Vec<Py<JID>>,
    #[pyo3(get)]
    pub failed_groups: Vec<(Py<JID>, u32)>,
}

impl UnlinkSubgroupsResult {
    pub fn from_inner(py: Python<'_>, value: WaUnlinkSubgroupsResult) -> PyResult<Self> {
        let unlinked_jids = value
            .unlinked_jids
            .into_iter()
            .map(|jid| Py::new(py, JID::from(jid)))
            .collect::<PyResult<Vec<_>>>()?;
        let failed_groups = value
            .failed_groups
            .into_iter()
            .map(|(jid, code)| Ok((Py::new(py, JID::from(jid))?, code)))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self {
            unlinked_jids,
            failed_groups,
        })
    }
}

#[pyclass]
pub struct GroupParticipant {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub phone_number: Option<Py<JID>>,
    #[pyo3(get)]
    pub is_admin: bool,
}

impl GroupParticipant {
    pub fn from_inner(py: Python<'_>, value: WaGroupParticipant) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(value.jid))?,
            phone_number: value
                .phone_number
                .map(|jid| Py::new(py, JID::from(jid)))
                .transpose()?,
            is_admin: value.is_admin,
        })
    }
}

#[pyclass]
pub struct GroupMetadata {
    #[pyo3(get)]
    pub id: Py<JID>,
    #[pyo3(get)]
    pub subject: String,
    #[pyo3(get)]
    pub participants: Vec<Py<GroupParticipant>>,
    #[pyo3(get)]
    pub addressing_mode: String,
    #[pyo3(get)]
    pub creator: Option<Py<JID>>,
    #[pyo3(get)]
    pub creation_time: Option<u64>,
    #[pyo3(get)]
    pub subject_time: Option<u64>,
    #[pyo3(get)]
    pub subject_owner: Option<Py<JID>>,
    #[pyo3(get)]
    pub description: Option<String>,
    #[pyo3(get)]
    pub description_id: Option<String>,
    #[pyo3(get)]
    pub is_locked: bool,
    #[pyo3(get)]
    pub is_announcement: bool,
    #[pyo3(get)]
    pub ephemeral_expiration: u32,
    #[pyo3(get)]
    pub membership_approval: bool,
    #[pyo3(get)]
    pub member_add_mode: Option<String>,
    #[pyo3(get)]
    pub member_link_mode: Option<String>,
    #[pyo3(get)]
    pub size: Option<u32>,
    #[pyo3(get)]
    pub is_parent_group: bool,
    #[pyo3(get)]
    pub parent_group_jid: Option<Py<JID>>,
    #[pyo3(get)]
    pub is_default_sub_group: bool,
    #[pyo3(get)]
    pub is_general_chat: bool,
    #[pyo3(get)]
    pub allow_non_admin_sub_group_creation: bool,
}

impl GroupMetadata {
    pub fn from_inner(py: Python<'_>, value: WaGroupMetadata) -> PyResult<Self> {
        let participants = value
            .participants
            .into_iter()
            .map(|participant| {
                let participant = GroupParticipant::from_inner(py, participant)?;
                Py::new(py, participant)
            })
            .collect::<PyResult<Vec<_>>>()?;

        let addressing_mode = match value.addressing_mode {
            AddressingMode::Pn => "pn".to_string(),
            AddressingMode::Lid => "lid".to_string(),
        };

        Ok(Self {
            id: Py::new(py, JID::from(value.id))?,
            subject: value.subject,
            participants,
            addressing_mode,
            creator: value.creator.map(|jid| Py::new(py, JID::from(jid))).transpose()?,
            creation_time: value.creation_time,
            subject_time: value.subject_time,
            subject_owner: value
                .subject_owner
                .map(|jid| Py::new(py, JID::from(jid)))
                .transpose()?,
            description: value.description,
            description_id: value.description_id,
            is_locked: value.is_locked,
            is_announcement: value.is_announcement,
            ephemeral_expiration: value.ephemeral_expiration,
            membership_approval: value.membership_approval,
            member_add_mode: value.member_add_mode.map(|mode| format!("{mode:?}")),
            member_link_mode: value.member_link_mode.map(|mode| format!("{mode:?}")),
            size: value.size,
            is_parent_group: value.is_parent_group,
            parent_group_jid: value
                .parent_group_jid
                .map(|jid| Py::new(py, JID::from(jid)))
                .transpose()?,
            is_default_sub_group: value.is_default_sub_group,
            is_general_chat: value.is_general_chat,
            allow_non_admin_sub_group_creation: value.allow_non_admin_sub_group_creation,
        })
    }

    pub fn get_group_type(&self) -> GroupType {
        if self.is_default_sub_group {
            GroupType::LinkedAnnouncementGroup
        } else if self.is_general_chat {
            GroupType::LinkedGeneralGroup
        } else if self.parent_group_jid.is_some() {
            GroupType::LinkedSubgroup
        } else if self.is_parent_group {
            GroupType::Community
        } else {
            GroupType::Default
        }
    }
}

#[pymethods]
impl GroupMetadata {
    #[getter]
    fn group_type(&self) -> GroupType {
        self.get_group_type()
    }
}
