#[pyclass]
pub struct EvStreamReplaced;
#[pyclass(from_py_object)]
#[derive(Clone)]
enum TempBanReason {
    SentToTooManyPeople,
    SentBlockedNyUser,
    CreateTooManyGroups,
    SentTooManySameMessage,
    Unknown,
}

#[pyclass]
pub struct EvTemporaryBan {
    #[pyo3(get)]
    code: TempBanReason,
    #[pyo3(get)]
    expires_in_seconds: u64,
    #[pyo3(get)]
    description: String,
}

impl EvTemporaryBan {
    pub fn from_wacore(value: wacore::types::events::TemporaryBan) -> Self {
        let code = match value.code {
            wacore::types::events::TempBanReason::SentToTooManyPeople => {
                TempBanReason::SentToTooManyPeople
            }
            wacore::types::events::TempBanReason::BlockedByUsers => {
                TempBanReason::SentBlockedNyUser
            }
            wacore::types::events::TempBanReason::CreatedTooManyGroups => {
                TempBanReason::CreateTooManyGroups
            }
            wacore::types::events::TempBanReason::SentTooManySameMessage => {
                TempBanReason::SentTooManySameMessage
            }
            wacore::types::events::TempBanReason::BroadcastList => {
                TempBanReason::Unknown
            }
            wacore::types::events::TempBanReason::Unknown(_) => TempBanReason::Unknown,
        };

        Self {
            code,
            expires_in_seconds: value.expire.num_seconds().max(0) as u64,
            description: value.code.to_string(),
        }
    }
}


#[pyclass]
pub struct EvConnectFailure {
    #[pyo3(get)]
    reason: String,
    #[pyo3(get)]
    message: String,
    inner: Option<wacore_binary::node::Node>,
    node: Option<Py<Node>>,
}
impl EvConnectFailure {
    pub fn new(reason: ConnectFailureReason, message: String, raw_node: Option<wacore_binary::node::Node>) -> Self {
        Self { reason: connect_failure_reason_to_string(&reason), message: message, inner: raw_node, node: None }
    }
}
#[pymethods]
impl EvConnectFailure {
    #[getter]
    fn node(&mut self, py: Python<'_>) -> PyResult<Option<Py<Node>>> {
        if let Some(ref node) = self.node {
            Ok(Some(node.clone_ref(py)))
        } else if let Some(raw_node) = self.inner.as_ref(){
            let node_instance = Node::from_node(raw_node);
            let py_node = Py::new(py, node_instance)?;
            self.node = Some(py_node.clone_ref(py));
            Ok(Some(py_node))
        } else {
            Err(pyo3::exceptions::PyAttributeError::new_err("ConnectFailure does not contain a node"))
        }
    }
}

#[pyclass]
pub struct EvStreamError{
    #[pyo3(get)]
    code: String,
    inner: Option<wacore_binary::node::Node>,
    node: Option<Py<Node>>

}
impl EvStreamError {
    pub fn new(code: String, raw: Option<wacore_binary::node::Node>) -> Self {
        Self { code, inner: raw, node: None }
    }
}
#[pymethods]
impl EvStreamError {
    #[getter]
    fn node(&mut self, py: Python<'_>) -> PyResult<Option<Py<Node>>> {
        if let Some(ref node) = self.node {
            Ok(Some(node.clone_ref(py)))
        } else if let Some(raw_node) = self.inner.as_ref(){
            let node_instance = Node::from_node(raw_node);
            let py_node = Py::new(py, node_instance)?;
            self.node = Some(py_node.clone_ref(py));
            Ok(Some(py_node))
        } else {
            Err(pyo3::exceptions::PyAttributeError::new_err("StreamError does not contain a node"))
        }
    }
}
#[pyclass]
pub struct MessageData {
    inner: Box<waproto::whatsapp::Message>,
    inner_message_info: Box<wacore::types::message::MessageInfo>,
    message_info: OnceLock<Py<MessageInfo>>,
    message_proto: OnceLock<Py<PyAny>>,
}
impl MessageData {
    pub fn new(inner: Box<waproto::whatsapp::Message>, inner_message_info: Box<wacore::types::message::MessageInfo>) -> Self {
        Self { inner: inner, inner_message_info: inner_message_info, message_info: OnceLock::new(), message_proto: OnceLock::new() }
    }
}
#[pymethods]
impl MessageData {
    #[getter]
    fn conversation(&self) -> Option<&str> {
        self.inner.conversation.as_deref()
    }
    #[getter]
    fn caption(&self) -> Option<&str> {
        self.inner.image_message.as_ref().and_then(|img| img.caption.as_deref())
            .or_else(|| self.inner.video_message.as_ref().and_then(|vid| vid.caption.as_deref()))
            .or_else(|| self.inner.document_message.as_ref().and_then(|doc| doc.caption.as_deref()))
    }
    fn get_extended_text_message(&self) -> Option<&str> {
        if let Some(etm) = self.inner.extended_text_message.as_ref() {
            etm.text.as_deref()
        } else {
            None
        }
    }
    fn get_text(&self) -> Option<&str> {
        self.inner.conversation.as_deref()
            .or_else(|| self.inner.extended_text_message.as_ref().and_then(|etm| etm.text.as_deref()))
    }
    #[getter]
    fn raw_proto(&mut self, py: Python) -> PyResult<Py<PyAny>> {
        match self.message_proto.get() {
            Some(ref proto) => Ok(proto.clone_ref(py)),
            None => {
                let mut buffer = Vec::new();
                self.inner
                    .as_ref()
                    .encode(&mut buffer)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

                let new_proto = parse_message_proto(py, &buffer)?;
                let out_proto = new_proto.clone_ref(py);
                self.message_proto.set(out_proto.clone_ref(py)).unwrap();
                Ok(out_proto)
            },
        }
    }
    fn __repr__(&self) -> String {
        format!("Message(conversation={:?}, caption={:?})", self.conversation(), self.caption())
    }
}     


#[pyclass]
pub struct EvMessage {
    pub inner: Box<waproto::whatsapp::Message>,
    pub inner_message_info: Box<wacore::types::message::MessageInfo>,
    pub data_cache: OnceLock<Py<MessageData>>,
}
impl EvMessage {
    pub fn new(inner: waproto::whatsapp::Message, message_info: WhatsappMessageInfo) -> Self {
        Self { inner: Box::new(inner), inner_message_info: Box::new(message_info), data_cache: OnceLock::new() }
    }
}
#[pymethods]
impl EvMessage {
    #[getter]
    fn data(&mut self, py: Python) -> PyResult<Py<MessageData>> {
        if let Some(ref data) = self.data_cache.get() {
            Ok(data.clone_ref(py))
        } else {
            let new_data = MessageData::new(Box::clone(&self.inner), Box::clone(&self.inner_message_info));
            let py_data = Py::new(py, new_data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}

pub fn connect_failure_reason_to_string(reason: &ConnectFailureReason) -> String {
    match reason {
        ConnectFailureReason::BadUserAgent => "BadUserAgent".into(),
        ConnectFailureReason::LoggedOut => "LoggedOut".into(),
        ConnectFailureReason::CatExpired => "CatExpired".into(),
        ConnectFailureReason::CatInvalid => "CatInvalid".into(),
        ConnectFailureReason::ClientOutdated => "ClientOutdated".into(),
        ConnectFailureReason::ClientUnknown => "ClientUnknown".into(),
        ConnectFailureReason::Generic => "Generic".into(),
        ConnectFailureReason::TempBanned => "TempBanned".into(),
        ConnectFailureReason::UnknownLogout => "Unknown".into(),
        ConnectFailureReason::MainDeviceGone => "MainDeviceGone".into(),
        ConnectFailureReason::NotFound => "NotFound".into(),
        ConnectFailureReason::ServiceUnavailable => "ServiceUnavailable".into(),
        ConnectFailureReason::InternalServerError => "InternalServerError".into(),
        ConnectFailureReason::Experimental => "Experimental".into(),
        ConnectFailureReason::Unknown(value) => format!("Unknown({})", value).into(), 
    }
}

#[pyclass]
pub struct EvArchiveUpdate {
    inner: Box<wacore::types::events::ArchiveUpdate>,
    data_cache: OnceLock<Py<EvArchiveUpdateData>>,
}

#[pyclass]
pub struct EvArchiveUpdateData {
    #[pyo3(get)]
    jid: JID,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    action: Box<waproto::whatsapp::sync_action_value::ArchiveChatAction>,
    #[pyo3(get)]
    from_full_sync: bool,
    action_cache: OnceLock<Py<PyAny>>,
}
impl EvArchiveUpdate {
    pub fn new(inner: wacore::types::events::ArchiveUpdate) -> Self {
        Self {
            inner: Box::new(inner),
            data_cache: OnceLock::new(),
        }
    }
}

impl From<wacore::types::events::ArchiveUpdate> for EvArchiveUpdate {
    fn from(event: wacore::types::events::ArchiveUpdate) -> Self {
        EvArchiveUpdate::new(event)
    }
}

#[pymethods]
impl EvArchiveUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<EvArchiveUpdateData>> {
        if let Some(cached) = self.data_cache.get() {
            Ok(cached.clone_ref(py))
        } else {
            let timestamp = PyDateTime::from_timestamp(py, (self.inner.timestamp.timestamp_millis() as f64) / 1000.0, None)?
                .into();
            let data = EvArchiveUpdateData {
                jid: self.inner.jid.clone().into(),
                timestamp,
                action: Box::new((*self.inner.action).clone()),
                from_full_sync: self.inner.from_full_sync,
                action_cache: OnceLock::new(),
            };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}

#[pymethods]
impl EvArchiveUpdateData {
    #[getter]
    fn action(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(cache) = self.action_cache.get() {
            Ok(cache.clone_ref(py))
        } else {
            let proto_instance = from_string_to_python_proto(
                py,
                get_proto_sync_action_value_from_string(py)?,
                self.action.as_ref().encode_to_vec().as_slice(),
            )?;
            self.action_cache.set(proto_instance.clone_ref(py)).ok();
            Ok(proto_instance)
        }
    }
}

#[pyclass]
pub struct EvDisappearingModeChangedData {
    #[pyo3(get)]
    from: JID,
    #[pyo3(get)]
    duration: u32,
    #[pyo3(get)]
    setting_timestamp: u64
}

#[pyclass]
pub struct EvDisappearingModeChanged {
    inner: Box<wacore::types::events::DisappearingModeChanged>,
    data_cache: OnceLock<Py<EvDisappearingModeChangedData>>,
}

impl EvDisappearingModeChanged {
    pub fn new(inner: wacore::types::events::DisappearingModeChanged) -> Self {
        Self {
            inner: Box::new(inner),
            data_cache: OnceLock::new(),
        }
    }
}

impl From<wacore::types::events::DisappearingModeChanged> for EvDisappearingModeChanged {
    fn from(event: wacore::types::events::DisappearingModeChanged) -> Self {
        EvDisappearingModeChanged::new(event)
    }
}

#[pymethods]
impl EvDisappearingModeChanged {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<EvDisappearingModeChangedData>> {
        if let Some(cached) = self.data_cache.get() {
            Ok(cached.clone_ref(py))
        } else {
            let data = EvDisappearingModeChangedData {
                from: self.inner.from.clone().into(),
                duration: self.inner.duration,
                setting_timestamp: self.inner.setting_timestamp,
            };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}

#[pyclass]
pub struct EvContactNumberChangedData {
    #[pyo3(get)]
    old_jid: JID,
    #[pyo3(get)]
    new_jid: JID,
    #[pyo3(get)]
    old_lid: Option<JID>,
    #[pyo3(get)]
    new_lid: Option<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
}

#[pyclass]
pub struct EvContactNumberChanged {
    inner: Box<wacore::types::events::ContactNumberChanged>,
    data_cache: OnceLock<Py<EvContactNumberChangedData>>,
}

impl EvContactNumberChanged {
    pub fn new(inner: wacore::types::events::ContactNumberChanged) -> Self {
        Self {
            inner: Box::new(inner),
            data_cache: OnceLock::new(),
        }
    }
}

impl From<wacore::types::events::ContactNumberChanged> for EvContactNumberChanged {
    fn from(event: wacore::types::events::ContactNumberChanged) -> Self {
        EvContactNumberChanged::new(event)
    }
}

#[pymethods]
impl EvContactNumberChanged {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<EvContactNumberChangedData>> {
        if let Some(cached) = self.data_cache.get() {
            Ok(cached.clone_ref(py))
        } else {
            let timestamp = PyDateTime::from_timestamp(
                py,
                (self.inner.timestamp.timestamp_millis() as f64) / 1000.0,
                None,
            )
            ?
            .into();
            let data = EvContactNumberChangedData {
                old_jid: self.inner.old_jid.clone().into(),
                new_jid: self.inner.new_jid.clone().into(),
                old_lid: self.inner.old_lid.clone().map(|j| j.into()),
                new_lid: self.inner.new_lid.clone().map(|j| j.into()),
                timestamp,
            };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}

#[pyclass]
pub struct EvContactSyncRequestedData {
    #[pyo3(get)]
    after: Option<Py<PyDateTime>>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
}

#[pyclass]
pub struct EvContactSyncRequested {
    inner: Box<wacore::types::events::ContactSyncRequested>,
    data_cache: OnceLock<Py<EvContactSyncRequestedData>>,
}

impl EvContactSyncRequested {
    pub fn new(inner: wacore::types::events::ContactSyncRequested) -> Self {
        Self {
            inner: Box::new(inner),
            data_cache: OnceLock::new(),
        }
    }
}

impl From<wacore::types::events::ContactSyncRequested> for EvContactSyncRequested {
    fn from(event: wacore::types::events::ContactSyncRequested) -> Self {
        EvContactSyncRequested::new(event)
    }
}

#[pymethods]
impl EvContactSyncRequested {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<EvContactSyncRequestedData>> {
        if let Some(cached) = self.data_cache.get() {
            Ok(cached.clone_ref(py))
        } else {
            let after = self
                .inner
                .after
                .map(|dt| PyDateTime::from_timestamp(py, (dt.timestamp_millis() as f64) / 1000.0, None).map(|v| v.into()))
                .transpose()?;
            let timestamp =
                PyDateTime::from_timestamp(py, (self.inner.timestamp.timestamp_millis() as f64) / 1000.0, None)
                    ?
                    .into();
            let data = EvContactSyncRequestedData { after, timestamp };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}

#[pyclass]
pub struct EvContactUpdatedData {
    #[pyo3(get)]
    jid: JID,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
}

#[pyclass]
pub struct EvContactUpdated {
    inner: Box<wacore::types::events::ContactUpdated>,
    data_cache: OnceLock<Py<EvContactUpdatedData>>,
}

impl EvContactUpdated {
    pub fn new(inner: wacore::types::events::ContactUpdated) -> Self {
        Self {
            inner: Box::new(inner),
            data_cache: OnceLock::new(),
        }
    }
}

impl From<wacore::types::events::ContactUpdated> for EvContactUpdated {
    fn from(event: wacore::types::events::ContactUpdated) -> Self {
        EvContactUpdated::new(event)
    }
}

#[pymethods]
impl EvContactUpdated {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<EvContactUpdatedData>> {
        if let Some(cached) = self.data_cache.get() {
            Ok(cached.clone_ref(py))
        } else {
            let timestamp = PyDateTime::from_timestamp(py, (self.inner.timestamp.timestamp_millis() as f64) / 1000.0, None)
                ?
                .into();
            let data = EvContactUpdatedData {
                jid: self.inner.jid.clone().into(),
                timestamp,
            };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}

#[pyclass]
pub struct EvStarUpdateData {
    #[pyo3(get)]
    chat_jid: JID,
    #[pyo3(get)]
    participant_jid: Option<JID>,
    #[pyo3(get)]
    message_id: String,
    #[pyo3(get)]
    from_me: bool,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    from_full_sync: bool,
    #[pyo3(get)]
    starred: Option<bool>,
}

#[pyclass]
pub struct EvStarUpdate {
    inner: Box<wacore::types::events::StarUpdate>,
    data_cache: OnceLock<Py<EvStarUpdateData>>,
}

impl EvStarUpdate {
    pub fn new(inner: wacore::types::events::StarUpdate) -> Self {
        Self {
            inner: Box::new(inner),
            data_cache: OnceLock::new(),
        }
    }
}

impl From<wacore::types::events::StarUpdate> for EvStarUpdate {
    fn from(event: wacore::types::events::StarUpdate) -> Self {
        EvStarUpdate::new(event)
    }
}

#[pymethods]
impl EvStarUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<EvStarUpdateData>> {
        if let Some(cached) = self.data_cache.get() {
            Ok(cached.clone_ref(py))
        } else {
            let timestamp = PyDateTime::from_timestamp(py, (self.inner.timestamp.timestamp_millis() as f64) / 1000.0, None)
                ?
                .into();
            let data = EvStarUpdateData {
                chat_jid: self.inner.chat_jid.clone().into(),
                participant_jid: self.inner.participant_jid.clone().map(|j| j.into()),
                message_id: self.inner.message_id.clone(),
                from_me: self.inner.from_me,
                timestamp,
                from_full_sync: self.inner.from_full_sync,
                starred: self.inner.action.starred,
            };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}

#[pyclass]
struct GroupParticipant {
    #[pyo3(get)]
    jid: JID,
    #[pyo3(get)]
    phone_number: Option<JID>
}

#[pyclass]
enum GroupNotificationAction {
    Add {
        participants: Vec<Py<GroupParticipant>>,
        reason: Option<String>,
    },
    Remove {
        participants: Vec<Py<GroupParticipant>>,
        reason: Option<String>,
    },
    Promote {
        participants: Vec<Py<GroupParticipant>>,
    },
    Demote {
        participants: Vec<Py<GroupParticipant>>,
    },
    Modify {
        participants: Vec<Py<GroupParticipant>>,
    },
    Subject {
        subject: String,
        subject_owner: Option<Py<JID>>,
        subject_timestamp: Option<Py<PyDateTime>>,
    },
    Description {
        id: String,
        description: Option<String>,
    },
    Locked {
        threshold: Option<String>,
    },
    Unlocked(),
    Announce(),
    NotAnnounce(),
    Ephemeral {
        expiration: u32,
        trigger: Option<u32>,
    },
    MembershipApprovalMode {
        enabled: bool,
    },
    MemberAddMode {
        mode: String,
    },
    NoFrequentlyForwarded(),
    FrequentlyForwardedOk(),
    Invite{
        code: String,
    },
    RevokeInvite(),
    GrowthLocked {
        expiration: u32,
        lock_type: String,
    },
    GrowthUnlocked(),
    Create {
        raw: Py<Node>,
    },
    Delete {
        reason: Option<String>,
    },
    Link {
        link_type: String,
        raw: Py<Node>,
    },
    Unlink {
        unlink_type: String,
        unlink_reason: Option<String>,
        raw: Py<Node>,
    },
    Unknown {
        tag: String,
    }

}

#[pyclass]
pub struct GroupUpdateData{
    #[pyo3(get)]
    group_jid: JID,
    #[pyo3(get)]
    participant: Option<JID>,
    #[pyo3(get)]
    participant_pn: Option<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    is_lid_addressing_mode: bool,
    #[pyo3(get)]
    action: Py<GroupNotificationAction>
}

#[pyclass]
pub struct EvGroupUpdate {
    inner: wacore::types::events::GroupUpdate,
    data_cache: OnceLock<Py<GroupUpdateData>>,
}
impl EvGroupUpdate {
    pub fn new(inner: wacore::types::events::GroupUpdate) -> Self {
        Self { inner, data_cache: OnceLock::new() }
    }
}
#[pymethods]
impl EvGroupUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<GroupUpdateData>> {
        if let Some(ref data) = self.data_cache.get() {
            Ok(data.clone_ref(py))
        } else {
            let participant = self.inner.participant.as_ref().map(|p| p.clone().into());
            let participant_pn = self.inner.participant_pn.as_ref().map(|pn| pn.clone().into());
            let timestamp = Python::attach(|py| PyDateTime::from_timestamp(py, self.inner.timestamp.timestamp() as f64, None).unwrap().into());

            let py_group_participants = |participants: &[wacore::stanza::groups::GroupParticipantInfo]| {
                participants
                    .iter()
                    .map(|p| {
                        let group_participant = GroupParticipant {
                            jid: p.jid.clone().into(),
                            phone_number: p.phone_number.as_ref().map(|pn| pn.clone().into()),
                        };
                        Py::new(py, group_participant).unwrap()
                    })
                    .collect::<Vec<_>>()
            };

            let action = match &self.inner.action {
                wacore::stanza::groups::GroupNotificationAction::Add { participants, reason } => {
                    let py_participants = py_group_participants(participants);
                    GroupNotificationAction::Add { participants: py_participants, reason: reason.clone() }
                },
                wacore::stanza::groups::GroupNotificationAction::Remove { participants, reason } => {
                    let py_participants = py_group_participants(participants);
                    GroupNotificationAction::Remove { participants: py_participants, reason: reason.clone() }
                },
                wacore::stanza::groups::GroupNotificationAction::Promote { participants } => {
                    let py_participants = py_group_participants(participants);
                    GroupNotificationAction::Promote { participants: py_participants }
                },
                wacore::stanza::groups::GroupNotificationAction::Demote { participants } => {
                    let py_participants = py_group_participants(participants);
                    GroupNotificationAction::Demote { participants: py_participants }
                },
                wacore::stanza::groups::GroupNotificationAction::Modify { participants } => {
                    let py_participants = py_group_participants(participants);
                    GroupNotificationAction::Modify { participants: py_participants }
                },
                wacore::stanza::groups::GroupNotificationAction::Subject { subject, subject_owner, subject_time } => {
                    let py_subject_owner = subject_owner
                        .as_ref()
                        .map(|o| Py::new(py, JID::from(o.clone())).unwrap());
                    let py_subject_timestamp = subject_time.map(|t| {
                        Python::attach(|py| PyDateTime::from_timestamp(py, t as f64, None).unwrap().into())
                    });
                    GroupNotificationAction::Subject { subject: subject.clone(), subject_owner: py_subject_owner, subject_timestamp: py_subject_timestamp }
                },
                wacore::stanza::groups::GroupNotificationAction::Description { id, description } => {
                    GroupNotificationAction::Description { id: id.clone(), description: description.clone() }
                },
                wacore::stanza::groups::GroupNotificationAction::Locked { threshold } => {
                    GroupNotificationAction::Locked { threshold: threshold.clone() }
                },
                wacore::stanza::groups::GroupNotificationAction::Unlocked => GroupNotificationAction::Unlocked(),
                wacore::stanza::groups::GroupNotificationAction::Announce => GroupNotificationAction::Announce(),
                wacore::stanza::groups::GroupNotificationAction::NotAnnounce => GroupNotificationAction::NotAnnounce(),
                wacore::stanza::groups::GroupNotificationAction::Ephemeral { expiration, trigger } => GroupNotificationAction::Ephemeral { expiration: *expiration, trigger: *trigger },
                wacore::stanza::groups::GroupNotificationAction::MembershipApprovalMode { enabled } => GroupNotificationAction::MembershipApprovalMode { enabled: *enabled },
                wacore::stanza::groups::GroupNotificationAction::MemberAddMode { mode } => GroupNotificationAction::MemberAddMode { mode: mode.clone() },
                wacore::stanza::groups::GroupNotificationAction::NoFrequentlyForwarded => GroupNotificationAction::NoFrequentlyForwarded(),
                wacore::stanza::groups::GroupNotificationAction::FrequentlyForwardedOk => GroupNotificationAction::FrequentlyForwardedOk(),
                wacore::stanza::groups::GroupNotificationAction::Invite { code } => GroupNotificationAction::Invite { code: code.clone() },
                wacore::stanza::groups::GroupNotificationAction::RevokeInvite => GroupNotificationAction::RevokeInvite(),
                wacore::stanza::groups::GroupNotificationAction::GrowthLocked { expiration, lock_type } => GroupNotificationAction::GrowthLocked { expiration: *expiration, lock_type: lock_type.clone() },
                wacore::stanza::groups::GroupNotificationAction::GrowthUnlocked => GroupNotificationAction::GrowthUnlocked(),
                wacore::stanza::groups::GroupNotificationAction::Create { raw } => {
                    let py_raw = Py::new(py, Node::from_node(raw)).unwrap();
                    GroupNotificationAction::Create { raw: py_raw }
                },
                wacore::stanza::groups::GroupNotificationAction::Delete { reason } => GroupNotificationAction::Delete { reason: reason.clone() },
                wacore::stanza::groups::GroupNotificationAction::Link { link_type, raw } => {
                    let py_raw = Py::new(py, Node::from_node(raw)).unwrap();
                    GroupNotificationAction::Link { link_type: link_type.clone(), raw: py_raw }
                },
                wacore::stanza::groups::GroupNotificationAction::Unlink { unlink_type, unlink_reason, raw } => {
                    let py_raw = Py::new(py, Node::from_node(raw)).unwrap();
                    GroupNotificationAction::Unlink { unlink_type: unlink_type.clone(), unlink_reason: unlink_reason.clone(), raw: py_raw }
                },
                wacore::stanza::groups::GroupNotificationAction::Unknown { tag } => GroupNotificationAction::Unknown { tag: tag.clone() },
            };
            let action = Py::new(py, action)?;
            let data = GroupUpdateData {
                group_jid: self.inner.group_jid.clone().into(),
                participant,
                participant_pn,
                timestamp,
                is_lid_addressing_mode: self.inner.is_lid_addressing_mode,
                action,
            };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}


#[pyclass]
struct ContactUpdateData {
    #[pyo3(get)]
    jid: JID,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    action_cache: OnceLock<Py<PyAny>>,
    action: Box<waproto::whatsapp::sync_action_value::ContactAction>,
    #[pyo3(get)]
    from_full_sync: bool,
}

#[pymethods]
impl ContactUpdateData {
    #[getter]
    fn action(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(cache) = self.action_cache.get() {
            Ok(cache.clone_ref(py))
        } else {
            let proto_instance = from_string_to_python_proto(py, get_proto_contact_action_proto_type(py)?, self.action.as_ref().encode_to_vec().as_slice())?;
            self.action_cache.set(proto_instance.clone_ref(py)).ok();
            Ok(proto_instance)
        }
    }
}

#[pyclass]
pub struct EvContactUpdate{
    inner: Box<wacore::types::events::ContactUpdate>,
    contact_cache: OnceLock<Py<ContactUpdateData>>,

}   

impl EvContactUpdate {
    pub fn new(inner: wacore::types::events::ContactUpdate) -> Self {
        Self {
            inner: Box::new(inner),
            contact_cache: OnceLock::new(),
        }
    }
}
#[pymethods]
impl EvContactUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<ContactUpdateData>> {
        if let Some(ref cache) = self.contact_cache.get() {
            Ok(cache.clone_ref(py))
        } else {
            let timestamp = Python::attach(|py| PyDateTime::from_timestamp(py, self.inner.timestamp.timestamp() as f64, None).unwrap().into());
            let data = ContactUpdateData {
                jid: self.inner.jid.clone().into(),
                timestamp,
                action: self.inner.action.clone(),
                from_full_sync: self.inner.from_full_sync,
                action_cache: OnceLock::new(),
            };
            let py_data = Py::new(py, data)?;
            self.contact_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}
impl From<wacore::types::events::ContactUpdate> for EvContactUpdate {
    fn from(event: wacore::types::events::ContactUpdate) -> Self {
        EvContactUpdate::new(event)
    }
}
