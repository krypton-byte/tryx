use pyo3::prelude::*;
use wacore::iq::privacy::{
    DisallowedListAction as WaDisallowedListAction,
    DisallowedListUpdate as WaDisallowedListUpdate,
    DisallowedListUserEntry as WaDisallowedListUserEntry,
    PrivacyCategory as WaPrivacyCategory,
    PrivacySetting as WaPrivacySetting,
    PrivacyValue as WaPrivacyValue,
};

use crate::types::JID;

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivacyCategory {
    Last = 0,
    Online = 1,
    Profile = 2,
    Status = 3,
    GroupAdd = 4,
    ReadReceipts = 5,
    CallAdd = 6,
    Messages = 7,
    DefenseMode = 8,
    Other = 9,
}

impl From<WaPrivacyCategory> for PrivacyCategory {
    fn from(value: WaPrivacyCategory) -> Self {
        match value {
            WaPrivacyCategory::Last => Self::Last,
            WaPrivacyCategory::Online => Self::Online,
            WaPrivacyCategory::Profile => Self::Profile,
            WaPrivacyCategory::Status => Self::Status,
            WaPrivacyCategory::GroupAdd => Self::GroupAdd,
            WaPrivacyCategory::ReadReceipts => Self::ReadReceipts,
            WaPrivacyCategory::CallAdd => Self::CallAdd,
            WaPrivacyCategory::Messages => Self::Messages,
            WaPrivacyCategory::DefenseMode => Self::DefenseMode,
            WaPrivacyCategory::Other(_) => Self::Other,
        }
    }
}

impl From<PrivacyCategory> for WaPrivacyCategory {
    fn from(value: PrivacyCategory) -> Self {
        match value {
            PrivacyCategory::Last => Self::Last,
            PrivacyCategory::Online => Self::Online,
            PrivacyCategory::Profile => Self::Profile,
            PrivacyCategory::Status => Self::Status,
            PrivacyCategory::GroupAdd => Self::GroupAdd,
            PrivacyCategory::ReadReceipts => Self::ReadReceipts,
            PrivacyCategory::CallAdd => Self::CallAdd,
            PrivacyCategory::Messages => Self::Messages,
            PrivacyCategory::DefenseMode => Self::DefenseMode,
            PrivacyCategory::Other => Self::Other("other".to_string()),
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivacyValue {
    All = 0,
    Contacts = 1,
    None_ = 2,
    ContactBlacklist = 3,
    MatchLastSeen = 4,
    Known = 5,
    Off = 6,
    OnStandard = 7,
    Other = 8,
}

impl From<WaPrivacyValue> for PrivacyValue {
    fn from(value: WaPrivacyValue) -> Self {
        match value {
            WaPrivacyValue::All => Self::All,
            WaPrivacyValue::Contacts => Self::Contacts,
            WaPrivacyValue::None => Self::None_,
            WaPrivacyValue::ContactBlacklist => Self::ContactBlacklist,
            WaPrivacyValue::MatchLastSeen => Self::MatchLastSeen,
            WaPrivacyValue::Known => Self::Known,
            WaPrivacyValue::Off => Self::Off,
            WaPrivacyValue::OnStandard => Self::OnStandard,
            WaPrivacyValue::Other(_) => Self::Other,
        }
    }
}

impl From<PrivacyValue> for WaPrivacyValue {
    fn from(value: PrivacyValue) -> Self {
        match value {
            PrivacyValue::All => Self::All,
            PrivacyValue::Contacts => Self::Contacts,
            PrivacyValue::None_ => Self::None,
            PrivacyValue::ContactBlacklist => Self::ContactBlacklist,
            PrivacyValue::MatchLastSeen => Self::MatchLastSeen,
            PrivacyValue::Known => Self::Known,
            PrivacyValue::Off => Self::Off,
            PrivacyValue::OnStandard => Self::OnStandard,
            PrivacyValue::Other => Self::Other("other".to_string()),
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisallowedListAction {
    Add = 0,
    Remove = 1,
}

impl From<WaDisallowedListAction> for DisallowedListAction {
    fn from(value: WaDisallowedListAction) -> Self {
        match value {
            WaDisallowedListAction::Add => Self::Add,
            WaDisallowedListAction::Remove => Self::Remove,
        }
    }
}

impl From<DisallowedListAction> for WaDisallowedListAction {
    fn from(value: DisallowedListAction) -> Self {
        match value {
            DisallowedListAction::Add => Self::Add,
            DisallowedListAction::Remove => Self::Remove,
        }
    }
}

#[pyclass]
pub struct PrivacySetting {
    #[pyo3(get)]
    pub category: PrivacyCategory,
    #[pyo3(get)]
    pub value: PrivacyValue,
}

impl From<WaPrivacySetting> for PrivacySetting {
    fn from(value: WaPrivacySetting) -> Self {
        Self {
            category: value.category.into(),
            value: value.value.into(),
        }
    }
}

#[pyclass]
pub struct DisallowedListUserEntry {
    #[pyo3(get, set)]
    pub action: DisallowedListAction,
    #[pyo3(get, set)]
    pub jid: Py<JID>,
    #[pyo3(get, set)]
    pub pn_jid: Option<Py<JID>>,
}

#[pymethods]
impl DisallowedListUserEntry {
    #[new]
    #[pyo3(signature = (action, jid, pn_jid=None))]
    fn new(action: DisallowedListAction, jid: Py<JID>, pn_jid: Option<Py<JID>>) -> Self {
        Self { action, jid, pn_jid }
    }
}

impl DisallowedListUserEntry {
    pub fn to_rust_entry(&self, py: Python<'_>) -> WaDisallowedListUserEntry {
        let jid = self.jid.bind(py).borrow().as_whatsapp_jid();
        let pn_jid = self
            .pn_jid
            .as_ref()
            .map(|value| value.bind(py).borrow().as_whatsapp_jid());

        WaDisallowedListUserEntry {
            action: self.action.into(),
            jid,
            pn_jid,
        }
    }
}

#[pyclass]
pub struct DisallowedListUpdate {
    #[pyo3(get, set)]
    pub dhash: String,
    #[pyo3(get, set)]
    pub users: Vec<Py<DisallowedListUserEntry>>,
}

#[pymethods]
impl DisallowedListUpdate {
    #[new]
    #[pyo3(signature = (dhash, users=Vec::new()))]
    fn new(dhash: String, users: Vec<Py<DisallowedListUserEntry>>) -> Self {
        Self { dhash, users }
    }
}

impl DisallowedListUpdate {
    pub fn to_rust_update(&self, py: Python<'_>) -> WaDisallowedListUpdate {
        let users = self
            .users
            .iter()
            .map(|user| user.bind(py).borrow().to_rust_entry(py))
            .collect::<Vec<_>>();

        WaDisallowedListUpdate {
            dhash: self.dhash.clone(),
            users,
        }
    }
}
