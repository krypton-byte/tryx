use pyo3::{Bound, PyResult, Python, pyclass, pymethods};

use crate::types::JID;
use crate::wacore::iq::groups::{
    CreateGroupOptions,
    GroupParticipantOptions,
    MemberAddMode,
    MemberLinkMode,
    MembershipApprovalMode,
};

#[pyclass]
pub struct GroupsHelpers;

#[pymethods]
impl GroupsHelpers {
    #[staticmethod]
    fn strip_invite_url(code: String) -> String {
        let value = code.trim().trim_end_matches('/');
        value
            .strip_prefix("https://chat.whatsapp.com/")
            .or_else(|| value.strip_prefix("http://chat.whatsapp.com/"))
            .unwrap_or(value)
            .to_string()
    }

    #[staticmethod]
    #[pyo3(signature = (jid, phone_number=None, privacy=None))]
    fn build_participant<'py>(
        py: Python<'py>,
        jid: pyo3::Py<JID>,
        phone_number: Option<pyo3::Py<JID>>,
        privacy: Option<Vec<u8>>,
    ) -> PyResult<Bound<'py, GroupParticipantOptions>> {
        pyo3::Py::new(
            py,
            GroupParticipantOptions {
                jid,
                phone_number,
                privacy,
            },
        )
            .map(|obj| obj.into_bound(py))
    }

    #[staticmethod]
    #[pyo3(signature = (
        subject,
        participants=Vec::new(),
        member_link_mode=Some(MemberLinkMode::AdminLink),
        member_add_mode=Some(MemberAddMode::AllMemberAdd),
        membership_approval_mode=Some(MembershipApprovalMode::Off),
        ephemeral_expiration=Some(0),
        is_parent=false,
        closed=false,
        allow_non_admin_sub_group_creation=false,
        create_general_chat=false
    ))]
    fn build_create_options<'py>(
        py: Python<'py>,
        subject: String,
        participants: Vec<pyo3::Py<GroupParticipantOptions>>,
        member_link_mode: Option<MemberLinkMode>,
        member_add_mode: Option<MemberAddMode>,
        membership_approval_mode: Option<MembershipApprovalMode>,
        ephemeral_expiration: Option<u32>,
        is_parent: bool,
        closed: bool,
        allow_non_admin_sub_group_creation: bool,
        create_general_chat: bool,
    ) -> PyResult<Bound<'py, CreateGroupOptions>> {
        pyo3::Py::new(
            py,
            CreateGroupOptions {
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
            },
        )
        .map(|obj| obj.into_bound(py))
    }
}
