use std::sync::OnceLock;

use prost::Message;
use pyo3::prelude::*;

use whatsapp_rust::{
    NewsletterMessage as WaNewsletterMessage,
    NewsletterMetadata as WaNewsletterMetadata,
    NewsletterReactionCount as WaNewsletterReactionCount,
    NewsletterRole as WaNewsletterRole,
    NewsletterState as WaNewsletterState,
    NewsletterVerification as WaNewsletterVerification,
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
            message_type: value.message_type,
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
