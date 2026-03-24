use pyo3::prelude::*;
use whatsapp_rust::{
    CreateGroupResult as WaCreateGroupResult,
    GroupCreateOptions as WaGroupCreateOptions,
    GroupParticipantOptions as WaGroupParticipantOptions,
    JoinGroupResult as WaJoinGroupResult,
    MemberAddMode as WaMemberAddMode,
    MemberLinkMode as WaMemberLinkMode,
    MembershipApprovalMode as WaMembershipApprovalMode,
    MembershipRequest as WaMembershipRequest,
    ParticipantChangeResponse as WaParticipantChangeResponse,
};

use crate::types::JID;

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberLinkMode {
    AdminLink = 0,
    AllMemberLink = 1,
}

impl From<WaMemberLinkMode> for MemberLinkMode {
    fn from(value: WaMemberLinkMode) -> Self {
        match value {
            WaMemberLinkMode::AdminLink => Self::AdminLink,
            WaMemberLinkMode::AllMemberLink => Self::AllMemberLink,
        }
    }
}

impl From<MemberLinkMode> for WaMemberLinkMode {
    fn from(value: MemberLinkMode) -> Self {
        match value {
            MemberLinkMode::AdminLink => Self::AdminLink,
            MemberLinkMode::AllMemberLink => Self::AllMemberLink,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberAddMode {
    AdminAdd = 0,
    AllMemberAdd = 1,
}

impl From<WaMemberAddMode> for MemberAddMode {
    fn from(value: WaMemberAddMode) -> Self {
        match value {
            WaMemberAddMode::AdminAdd => Self::AdminAdd,
            WaMemberAddMode::AllMemberAdd => Self::AllMemberAdd,
        }
    }
}

impl From<MemberAddMode> for WaMemberAddMode {
    fn from(value: MemberAddMode) -> Self {
        match value {
            MemberAddMode::AdminAdd => Self::AdminAdd,
            MemberAddMode::AllMemberAdd => Self::AllMemberAdd,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembershipApprovalMode {
    Off = 0,
    On = 1,
}

impl From<WaMembershipApprovalMode> for MembershipApprovalMode {
    fn from(value: WaMembershipApprovalMode) -> Self {
        match value {
            WaMembershipApprovalMode::Off => Self::Off,
            WaMembershipApprovalMode::On => Self::On,
        }
    }
}

impl From<MembershipApprovalMode> for WaMembershipApprovalMode {
    fn from(value: MembershipApprovalMode) -> Self {
        match value {
            MembershipApprovalMode::Off => Self::Off,
            MembershipApprovalMode::On => Self::On,
        }
    }
}

#[pyclass]
pub struct GroupParticipantOptions {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub phone_number: Option<Py<JID>>,
    #[pyo3(get)]
    pub privacy: Option<Vec<u8>>,
}

#[pymethods]
impl GroupParticipantOptions {
    #[new]
    #[pyo3(signature = (jid, phone_number=None, privacy=None))]
    fn new(jid: Py<JID>, phone_number: Option<Py<JID>>, privacy: Option<Vec<u8>>) -> Self {
        Self {
            jid,
            phone_number,
            privacy,
        }
    }
}

impl GroupParticipantOptions {
    pub fn to_rust(&self, py: Python<'_>) -> WaGroupParticipantOptions {
        WaGroupParticipantOptions {
            jid: self.jid.bind(py).borrow().as_whatsapp_jid(),
            phone_number: self
                .phone_number
                .as_ref()
                .map(|v| v.bind(py).borrow().as_whatsapp_jid()),
            privacy: self.privacy.clone(),
        }
    }
}

#[pyclass]
pub struct CreateGroupOptions {
    #[pyo3(get, set)]
    pub subject: String,
    #[pyo3(get, set)]
    pub participants: Vec<Py<GroupParticipantOptions>>,
    #[pyo3(get, set)]
    pub member_link_mode: Option<MemberLinkMode>,
    #[pyo3(get, set)]
    pub member_add_mode: Option<MemberAddMode>,
    #[pyo3(get, set)]
    pub membership_approval_mode: Option<MembershipApprovalMode>,
    #[pyo3(get, set)]
    pub ephemeral_expiration: Option<u32>,
    #[pyo3(get, set)]
    pub is_parent: bool,
    #[pyo3(get, set)]
    pub closed: bool,
    #[pyo3(get, set)]
    pub allow_non_admin_sub_group_creation: bool,
    #[pyo3(get, set)]
    pub create_general_chat: bool,
}

#[pymethods]
impl CreateGroupOptions {
    #[new]
    #[pyo3(signature = (
        subject,
        participants = Vec::new(),
        member_link_mode = Some(MemberLinkMode::AdminLink),
        member_add_mode = Some(MemberAddMode::AllMemberAdd),
        membership_approval_mode = Some(MembershipApprovalMode::Off),
        ephemeral_expiration = Some(0),
        is_parent = false,
        closed = false,
        allow_non_admin_sub_group_creation = false,
        create_general_chat = false
    ))]
    fn new(
        subject: String,
        participants: Vec<Py<GroupParticipantOptions>>,
        member_link_mode: Option<MemberLinkMode>,
        member_add_mode: Option<MemberAddMode>,
        membership_approval_mode: Option<MembershipApprovalMode>,
        ephemeral_expiration: Option<u32>,
        is_parent: bool,
        closed: bool,
        allow_non_admin_sub_group_creation: bool,
        create_general_chat: bool,
    ) -> Self {
        Self {
            subject,
            participants,
            member_link_mode,
            member_add_mode,
            membership_approval_mode,
            ephemeral_expiration,
            is_parent,
            closed,
            allow_non_admin_sub_group_creation,
            create_general_chat,
        }
    }
}

impl CreateGroupOptions {
    pub fn to_rust_options(&self, py: Python<'_>) -> WaGroupCreateOptions {
        WaGroupCreateOptions {
            subject: self.subject.clone(),
            participants: self
                .participants
                .iter()
                .map(|item| item.bind(py).borrow().to_rust(py))
                .collect(),
            member_link_mode: self.member_link_mode.map(Into::into),
            member_add_mode: self.member_add_mode.map(Into::into),
            membership_approval_mode: self.membership_approval_mode.map(Into::into),
            ephemeral_expiration: self.ephemeral_expiration,
            is_parent: self.is_parent,
            closed: self.closed,
            allow_non_admin_sub_group_creation: self.allow_non_admin_sub_group_creation,
            create_general_chat: self.create_general_chat,
        }
    }
}

#[pyclass]
pub struct CreateGroupResult {
    #[pyo3(get)]
    pub gid: Py<JID>,
}

impl CreateGroupResult {
    pub fn from_inner(py: Python<'_>, value: WaCreateGroupResult) -> PyResult<Self> {
        Ok(Self {
            gid: Py::new(py, JID::from(value.gid))?,
        })
    }
}

#[pyclass]
pub struct JoinGroupResult {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub pending_approval: bool,
}

impl JoinGroupResult {
    pub fn from_inner(py: Python<'_>, value: WaJoinGroupResult) -> PyResult<Self> {
        match value {
            WaJoinGroupResult::Joined(jid) => Ok(Self {
                jid: Py::new(py, JID::from(jid))?,
                pending_approval: false,
            }),
            WaJoinGroupResult::PendingApproval(jid) => Ok(Self {
                jid: Py::new(py, JID::from(jid))?,
                pending_approval: true,
            }),
        }
    }
}

#[pyclass]
pub struct ParticipantChangeResponse {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub status: Option<String>,
    #[pyo3(get)]
    pub error: Option<String>,
}

impl ParticipantChangeResponse {
    pub fn from_inner(py: Python<'_>, value: WaParticipantChangeResponse) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(value.jid))?,
            status: value.status,
            error: value.error,
        })
    }
}

#[pyclass]
pub struct MembershipRequest {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub request_time: Option<u64>,
}

impl MembershipRequest {
    pub fn from_inner(py: Python<'_>, value: WaMembershipRequest) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(value.jid))?,
            request_time: value.request_time,
        })
    }
}

#[pyclass]
pub struct GroupInfo {
    #[pyo3(get)]
    pub participants: Vec<Py<JID>>,
    #[pyo3(get)]
    pub addressing_mode: String,
    #[pyo3(get)]
    pub lid_to_pn_map: Vec<(String, Py<JID>)>,
}

impl GroupInfo {
    pub fn from_inner(py: Python<'_>, value: wacore::client::context::GroupInfo) -> PyResult<Self> {
        let lid_to_pn_map = value
            .lid_to_pn_map()
            .iter()
            .map(|(lid, jid)| Ok((lid.clone(), Py::new(py, JID::from(jid.clone()))?)))
            .collect::<PyResult<Vec<_>>>()?;

        let participants = value
            .participants
            .into_iter()
            .map(|jid| Py::new(py, JID::from(jid)))
            .collect::<PyResult<Vec<_>>>()?;

        let addressing_mode = match value.addressing_mode {
            wacore::types::message::AddressingMode::Pn => "pn".to_string(),
            wacore::types::message::AddressingMode::Lid => "lid".to_string(),
        };

        Ok(Self {
            participants,
            addressing_mode,
            lid_to_pn_map,
        })
    }
}
