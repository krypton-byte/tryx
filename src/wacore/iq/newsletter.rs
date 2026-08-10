use std::sync::OnceLock;

use buffa::Message;
use pyo3::prelude::*;

use whatsapp_rust::{
    NewsletterMessage as WaNewsletterMessage,
    NewsletterMetadata as WaNewsletterMetadata,
    NewsletterReactionCount as WaNewsletterReactionCount,
    NewsletterRole as WaNewsletterRole,
    NewsletterState as WaNewsletterState,
    NewsletterVerification as WaNewsletterVerification,
    NewsletterAdminInfo as WaNewsletterAdminInfo,
    NewsletterAdminProfile as WaNewsletterAdminProfile,
    NewsletterFollower as WaNewsletterFollower,
};

use crate::events::proto_cache::parse_message_proto;
use crate::types::JID;

#[pyclass(eq, eq_int, skip_from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewsletterVerification {
    Verified = 0,
    Unverified = 1,
}

impl From<WaNewsletterVerification> for NewsletterVerification {
    fn from(value: WaNewsletterVerification) -> Self {
        match value {
            WaNewsletterVerification::Verified => Self::Verified,
            WaNewsletterVerification::Unverified => Self::Unverified,
            _ => Self::Unverified,
        }
    }
}

#[pyclass(eq, eq_int, skip_from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewsletterState {
    Active = 0,
    Suspended = 1,
    Geosuspended = 2,
}

impl From<WaNewsletterState> for NewsletterState {
    fn from(value: WaNewsletterState) -> Self {
        match value {
            WaNewsletterState::Active => Self::Active,
            WaNewsletterState::Suspended => Self::Suspended,
            WaNewsletterState::Geosuspended => Self::Geosuspended,
            _ => Self::Active,
        }
    }
}

#[pyclass(eq, eq_int, skip_from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewsletterRole {
    Owner = 0,
    Admin = 1,
    Subscriber = 2,
    Guest = 3,
}

impl From<WaNewsletterRole> for NewsletterRole {
    fn from(value: WaNewsletterRole) -> Self {
        match value {
            WaNewsletterRole::Owner => Self::Owner,
            WaNewsletterRole::Admin => Self::Admin,
            WaNewsletterRole::Subscriber => Self::Subscriber,
            WaNewsletterRole::Guest => Self::Guest,
            _ => Self::Guest,
        }
    }
}

#[pyclass]
pub struct NewsletterReactionCount {
    #[pyo3(get)]
    pub code: String,
    #[pyo3(get)]
    pub count: u64,
}

impl From<WaNewsletterReactionCount> for NewsletterReactionCount {
    fn from(value: WaNewsletterReactionCount) -> Self {
        Self {
            code: value.code,
            count: value.count,
        }
    }
}

#[pyclass]
pub struct NewsletterMetadata {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub description: Option<String>,
    #[pyo3(get)]
    pub subscriber_count: u64,
    #[pyo3(get)]
    pub verification: NewsletterVerification,
    #[pyo3(get)]
    pub state: NewsletterState,
    #[pyo3(get)]
    pub picture_url: Option<String>,
    #[pyo3(get)]
    pub preview_url: Option<String>,
    #[pyo3(get)]
    pub invite_code: Option<String>,
    #[pyo3(get)]
    pub role: Option<NewsletterRole>,
    #[pyo3(get)]
    pub creation_time: Option<u64>,
}

#[pyclass]
pub struct NewsletterAdminProfile {
    #[pyo3(get)] pub id: Option<String>,
    #[pyo3(get)] pub name: String,
    #[pyo3(get)] pub picture_id: Option<String>,
    #[pyo3(get)] pub picture_direct_path: Option<String>,
}

impl From<WaNewsletterAdminProfile> for NewsletterAdminProfile {
    fn from(value: WaNewsletterAdminProfile) -> Self {
        Self { id: value.id, name: value.name, picture_id: value.picture_id, picture_direct_path: value.picture_direct_path }
    }
}

#[pyclass]
pub struct NewsletterAdminInfo {
    #[pyo3(get)] pub admin_count: Option<u32>,
    #[pyo3(get)] pub admin_profile: Option<Py<NewsletterAdminProfile>>,
    #[pyo3(get)] pub admin_profiles_enabled: Option<bool>,
}

impl NewsletterAdminInfo {
    pub fn from_inner(py: Python<'_>, value: WaNewsletterAdminInfo) -> PyResult<Self> {
        Ok(Self {
            admin_count: value.admin_count,
            admin_profile: value.admin_profile.map(|p| Py::new(py, NewsletterAdminProfile::from(p))).transpose()?,
            admin_profiles_enabled: value.admin_profiles_enabled,
        })
    }
}

#[pyclass]
pub struct NewsletterFollower {
    #[pyo3(get)] pub jid: Py<JID>,
    #[pyo3(get)] pub phone_jid: Option<Py<JID>>,
    #[pyo3(get)] pub display_name: Option<String>,
    #[pyo3(get)] pub username: Option<String>,
    #[pyo3(get)] pub role: Option<NewsletterRole>,
    #[pyo3(get)] pub follow_time: Option<u64>,
    #[pyo3(get)] pub admin_profile: Option<Py<NewsletterAdminProfile>>,
}

impl NewsletterFollower {
    pub fn from_inner(py: Python<'_>, value: WaNewsletterFollower) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(value.jid))?,
            phone_jid: value.phone_jid.map(|j| Py::new(py, JID::from(j))).transpose()?,
            display_name: value.display_name, username: value.username,
            role: value.role.map(Into::into), follow_time: value.follow_time,
            admin_profile: value.admin_profile.map(|p| Py::new(py, NewsletterAdminProfile::from(p))).transpose()?,
        })
    }
}

impl NewsletterMetadata {
    pub fn from_inner(py: Python<'_>, value: WaNewsletterMetadata) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(value.jid))?,
            name: value.name,
            description: value.description,
            subscriber_count: value.subscriber_count,
            verification: value.verification.into(),
            state: value.state.into(),
            picture_url: value.picture_url,
            preview_url: value.preview_url,
            invite_code: value.invite_code,
            role: value.role.map(Into::into),
            creation_time: value.creation_time,
        })
    }
}

#[pyclass]
pub struct NewsletterMessage {
    #[pyo3(get)]
    pub server_id: u64,
    #[pyo3(get)]
    pub timestamp: u64,
    #[pyo3(get)]
    pub message_type: String,
    #[pyo3(get)]
    pub is_sender: bool,
    #[pyo3(get)]
    pub reactions: Vec<Py<NewsletterReactionCount>>,
    message_inner: Option<waproto::whatsapp::Message>,
    message_cache: OnceLock<Option<Py<PyAny>>>,
}

impl NewsletterMessage {
    pub fn from_inner(py: Python<'_>, value: WaNewsletterMessage) -> PyResult<Self> {
        let reactions = value
            .reactions
            .into_iter()
            .map(|item| Py::new(py, NewsletterReactionCount::from(item)))
            .collect::<PyResult<Vec<_>>>()?;

        Ok(Self {
            server_id: value.server_id,
            timestamp: value.timestamp,
            message_type: value.message_type.as_str().to_string(),
            is_sender: value.is_sender,
            reactions,
            message_inner: value.message,
            message_cache: OnceLock::new(),
        })
    }
}

#[pymethods]
impl NewsletterMessage {
    #[getter]
    fn message(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        if let Some(cached) = self.message_cache.get() {
            return Ok(cached.as_ref().map(|obj| obj.clone_ref(py)));
        }

        let parsed = match &self.message_inner {
            Some(message) => Some(parse_message_proto(py, message.encode_to_vec().as_slice())?),
            None => None,
        };
        let _ = self.message_cache.set(parsed);

        Ok(self
            .message_cache
            .get()
            .and_then(|cached| cached.as_ref().map(|obj| obj.clone_ref(py))))
    }
}
