#[pyclass]
pub struct DeviceProp {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub manufacturer: String,
    #[pyo3(get)]
    pub model: String,
    #[pyo3(get)]
    pub os_version: String,
}

#[pyclass]
pub struct EvIdentityChange {
    #[pyo3(get)]
    pub user: Py<JID>,
    #[pyo3(get)]
    pub lid_user: Option<Py<JID>>,
    #[pyo3(get)]
    pub implicit: bool,
}

impl EvIdentityChange {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::IdentityChange) -> PyResult<Self> {
        Ok(Self {
            user: Py::new(py, JID::from(ev.user.clone()))?,
            lid_user: ev.lid_user.clone().map(|j| Py::new(py, JID::from(j))).transpose()?,
            implicit: ev.implicit,
        })
    }
}

#[pyclass]
pub struct EvLockChatUpdate {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub locked: Option<bool>,
    #[pyo3(get)]
    pub timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pub from_full_sync: bool,
}

impl EvLockChatUpdate {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::LockChatUpdate) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(ev.jid.clone()))?,
            locked: ev.action.locked,
            timestamp: timestamp_to_py_datetime(py, ev.timestamp)?,
            from_full_sync: ev.from_full_sync,
        })
    }
}

#[pyclass]
pub struct EvFavoritesUpdate {
    #[pyo3(get)]
    pub favorite_ids: Vec<String>,
    #[pyo3(get)]
    pub timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pub from_full_sync: bool,
}

impl EvFavoritesUpdate {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::FavoritesUpdate) -> PyResult<Self> {
        Ok(Self {
            favorite_ids: ev.action.favorites.iter().filter_map(|f| f.id.clone()).collect(),
            timestamp: timestamp_to_py_datetime(py, ev.timestamp)?,
            from_full_sync: ev.from_full_sync,
        })
    }
}

#[pyclass]
pub struct EvStatusPrivacyUpdate {
    #[pyo3(get)]
    pub mode: Option<i32>,
    #[pyo3(get)]
    pub timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pub from_full_sync: bool,
}

impl EvStatusPrivacyUpdate {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::StatusPrivacyUpdate) -> PyResult<Self> {
        Ok(Self {
            mode: ev.action.mode.map(|m| m.to_i32()),
            timestamp: timestamp_to_py_datetime(py, ev.timestamp)?,
            from_full_sync: ev.from_full_sync,
        })
    }
}

#[pyclass]
pub struct EvUnarchiveChatsSettingUpdate {
    #[pyo3(get)]
    pub unarchive_chats: bool,
    #[pyo3(get)]
    pub timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pub from_full_sync: bool,
}

impl EvUnarchiveChatsSettingUpdate {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::UnarchiveChatsSettingUpdate) -> PyResult<Self> {
        Ok(Self {
            unarchive_chats: ev.unarchive_chats,
            timestamp: timestamp_to_py_datetime(py, ev.timestamp)?,
            from_full_sync: ev.from_full_sync,
        })
    }
}

#[pyclass]
pub struct EvDisableLinkPreviewsUpdate {
    #[pyo3(get)]
    pub previews_disabled: bool,
    #[pyo3(get)]
    pub timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pub from_full_sync: bool,
}

impl EvDisableLinkPreviewsUpdate {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::DisableLinkPreviewsUpdate) -> PyResult<Self> {
        Ok(Self {
            previews_disabled: ev.previews_disabled,
            timestamp: timestamp_to_py_datetime(py, ev.timestamp)?,
            from_full_sync: ev.from_full_sync,
        })
    }
}

#[pyclass]
pub struct EvContactRemoved {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pub from_full_sync: bool,
}

impl EvContactRemoved {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::ContactRemoved) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(ev.jid.clone()))?,
            timestamp: timestamp_to_py_datetime(py, ev.timestamp)?,
            from_full_sync: ev.from_full_sync,
        })
    }
}

#[pyclass]
pub struct EvOfflineSyncInterrupted {
    #[pyo3(get)]
    pub total: i32,
    #[pyo3(get)]
    pub delivered: i32,
}

impl From<&wacore::types::events::OfflineSyncInterrupted> for EvOfflineSyncInterrupted {
    fn from(ev: &wacore::types::events::OfflineSyncInterrupted) -> Self {
        Self {
            total: ev.total,
            delivered: ev.delivered,
        }
    }
}

#[pyclass]
pub struct EvClientExpirationChanged {
    #[pyo3(get)]
    pub expires_at: Option<i64>,
    #[pyo3(get)]
    pub version: (u32, u32, u32),
    #[pyo3(get)]
    pub withdrawn: bool,
}

impl From<&wacore::types::events::ClientExpirationChanged> for EvClientExpirationChanged {
    fn from(ev: &wacore::types::events::ClientExpirationChanged) -> Self {
        Self {
            expires_at: ev.expires_at,
            version: ev.version,
            withdrawn: ev.withdrawn,
        }
    }
}

#[pyclass]
pub struct EvServerAck {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub class: Option<String>,
    #[pyo3(get)]
    pub from_: Option<Py<JID>>,
    #[pyo3(get)]
    pub error: Option<String>,
}

impl EvServerAck {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::ServerAck) -> PyResult<Self> {
        Ok(Self {
            id: ev.id.clone(),
            class: ev.class.clone(),
            from_: ev.from.clone().map(|j| Py::new(py, JID::from(j))).transpose()?,
            error: ev.error.clone(),
        })
    }
}

#[pyclass]
pub struct EvAppStateSyncFailed {
    #[pyo3(get)]
    pub fatal: Vec<String>,
    #[pyo3(get)]
    pub retryable: Vec<String>,
    #[pyo3(get)]
    pub skipped: Vec<String>,
    #[pyo3(get)]
    pub connected: bool,
}

impl From<&wacore::types::events::AppStateSyncFailed> for EvAppStateSyncFailed {
    fn from(ev: &wacore::types::events::AppStateSyncFailed) -> Self {
        Self {
            fatal: ev.fatal.clone(),
            retryable: ev.retryable.clone(),
            skipped: ev.skipped.clone(),
            connected: ev.connected,
        }
    }
}

#[pyclass]
pub struct EvMexNotification {
    #[pyo3(get)]
    pub op_name: String,
    #[pyo3(get)]
    pub from_: Option<Py<JID>>,
    #[pyo3(get)]
    pub stanza_id: Option<String>,
    #[pyo3(get)]
    pub offline: Option<String>,
    #[pyo3(get)]
    pub payload_json: String,
}

impl EvMexNotification {
    pub fn from_event(py: Python<'_>, ev: &wacore::types::events::MexNotification) -> PyResult<Self> {
        Ok(Self {
            op_name: ev.op_name.clone(),
            from_: ev.from.clone().map(|j| Py::new(py, JID::from(j))).transpose()?,
            stanza_id: ev.stanza_id.clone(),
            offline: ev.offline.clone(),
            payload_json: ev.payload.to_string(),
        })
    }
}