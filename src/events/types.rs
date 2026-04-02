use std::sync::OnceLock;

use prost::Message as ProstMessage;
use pyo3::{Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3::types::{PyDateTime};
use chrono::{DateTime, Utc};
use wacore::types::events::QrScannedWithoutMultidevice;
use whatsapp_rust::{Jid, types::events::{ConnectFailureReason, LoggedOut as WhatsAppLoggedOut }};
use whatsapp_rust::types::message::{MessageInfo as WhatsappMessageInfo};
use crate::types::{JID, MessageInfo, MessageSource};
use crate::events::proto_cache::{
    parse_message_proto,
    parse_proto_bytes as from_string_to_python_proto,
    proto_contact_action as get_proto_contact_action_proto_type,
    proto_delete_chat_action as get_proto_delete_chat_action_proto_type,
    proto_delete_message_for_me_action as get_proto_delete_message_for_me_action_proto_type,
    proto_history_sync as get_proto_history_sync_proto_type,
    proto_lazy_conversation as get_lazy_conversation_proto_type,
    proto_mark_chat_as_read_action as get_proto_mark_chat_as_read_action_proto_type,
    proto_mute_action as get_proto_mute_action_proto_type,
    proto_sync_action_value as get_proto_sync_action_value_from_string,
};
use crate::wacore::node::Node;
use crate::wacore::stanza::{BusinessSubscription, KeyIndexInfo};

#[pyclass]
pub struct EvConnected;

#[pyclass]
pub struct EvDisconnected;

#[pyclass]
pub struct EvLoggedOut{
    #[pyo3(get)]
    on_connect: bool,
    #[pyo3(get)]
    reason: String,
}
impl EvLoggedOut {
    pub fn new(logout: WhatsAppLoggedOut) -> Self {
        Self { on_connect: logout.on_connect, reason: connect_failure_reason_to_string(&logout.reason) }
    }
}

#[pyclass]
pub struct PairSuccessData {
    #[pyo3(get)]
    id: Py<JID>,
    #[pyo3(get)]
    lid: Py<JID>,
    #[pyo3(get)]
    business_name: String,
    #[pyo3(get)]
    platform: String,
}
#[pyclass]
pub struct EvPairSuccess {
    inner: Box<wacore::types::events::PairSuccess>,
    data_cached: OnceLock<Py<PairSuccessData>>,
}
impl EvPairSuccess {
    pub fn new(inner: wacore::types::events::PairSuccess) -> Self {
        Self {
            inner: Box::new(inner),
            data_cached: OnceLock::new(),
        }
    }
}
#[pymethods]
impl EvPairSuccess {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<PairSuccessData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let new_data = PairSuccessData { id: Py::new(py, JID::from(self.inner.id.clone())).unwrap(), lid: Py::new(py, JID::from(self.inner.lid.clone())).unwrap(), business_name: self.inner.business_name.clone(), platform: self.inner.platform.clone() };
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
     }
}
impl From<wacore::types::events::PairSuccess> for EvPairSuccess {
    fn from(event: wacore::types::events::PairSuccess) -> Self {
        EvPairSuccess::new(event)
    }
}
#[pyclass]
pub struct EvPairError {
    #[pyo3(get)]
    id: Py<JID>,
    #[pyo3(get)]
    lid: Py<JID>,
    #[pyo3(get)]
    business_name: String,
    #[pyo3(get)]
    platform: String,
    #[pyo3(get)]
    error: String,
}
impl EvPairError {
    pub fn new(id: JID, lid: JID, business_name: String, platform: String, error: String) -> Self {
        Python::attach(|py|{
            Self { id: Py::new(py, id).unwrap(), lid: Py::new(py, lid).unwrap(), business_name, platform, error }
        })
    }
}
#[pyclass]
pub struct EvPairingQrCode {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    timeout: u64,
}
impl EvPairingQrCode {
    pub fn new(code: String, timeout: u64) -> Self {
        Self { code, timeout }
    }

}

#[pyclass]
pub struct EvPairingCode {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    timeout: u64,
}

impl EvPairingCode {
    pub fn new(code: String, timeout: u64) -> Self {
        Self { code, timeout }
    }
}

#[pyclass]
pub struct EvQrScannedWithoutMultidevice;

impl From<QrScannedWithoutMultidevice> for EvQrScannedWithoutMultidevice {
    fn from(_: QrScannedWithoutMultidevice) -> Self {
        EvQrScannedWithoutMultidevice{}
    }
}


#[pyclass]
pub struct EvClientOutDated;

#[pyclass]
pub enum ReceiptType {
    Delivered,
    Sender,
    Retry,
    Read,
    ReadSelf,
    Played,
    PlayedSelf,
    ServerError,
    Inactive,
    PeerMsg,
    HistorySync,
    EncRekeyRetry,
    Other
}

#[pyclass]
pub struct EvReceipt {
    inner: Box<wacore::types::message::MessageSource>,
    source: Option<pyo3::Py<MessageSource>>,
    #[pyo3(get)]
    message_ids: Vec<String>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    receipt_type: Py<ReceiptType>,
    #[pyo3(get)]
    message_sender: Py<JID>
}

impl EvReceipt {
    pub fn new(inner: wacore::types::message::MessageSource, message_ids: Vec<String>, timestamp: DateTime<Utc>, r#type: wacore::types::presence::ReceiptType, message_sender: Jid) -> Py<Self> {
        let receipt_type = match r#type {
            wacore::types::presence::ReceiptType::Delivered => ReceiptType::Delivered,
            wacore::types::presence::ReceiptType::Sender => ReceiptType::Sender,
            wacore::types::presence::ReceiptType::Retry => ReceiptType::Retry,
            wacore::types::presence::ReceiptType::Read => ReceiptType::Read,
            wacore::types::presence::ReceiptType::ReadSelf => ReceiptType::ReadSelf,
            wacore::types::presence::ReceiptType::Played => ReceiptType::Played,
            wacore::types::presence::ReceiptType::PlayedSelf => ReceiptType::PlayedSelf,
            wacore::types::presence::ReceiptType::ServerError => ReceiptType::ServerError,
            wacore::types::presence::ReceiptType::Inactive => ReceiptType::Inactive,
            wacore::types::presence::ReceiptType::PeerMsg => ReceiptType::PeerMsg,
            wacore::types::presence::ReceiptType::HistorySync => ReceiptType::HistorySync,
            wacore::types::presence::ReceiptType::EncRekeyRetry => ReceiptType::EncRekeyRetry,
            wacore::types::presence::ReceiptType::Other(_) => ReceiptType::Other,
        };
        Python::attach(|py| {
            pyo3::Py::new(py, Self {
                inner: Box::new(inner),
                source: None,
                message_ids,
                timestamp: PyDateTime::from_timestamp(py, timestamp.naive_utc().and_utc().timestamp_millis() as f64 / 1000.0, None).unwrap().into(),
                receipt_type: Py::new(py, receipt_type).unwrap(),
                message_sender: Py::new(py, JID::from(message_sender.clone())).unwrap(),
            })
        })
        .unwrap()
    }
}

#[pymethods]
impl EvReceipt {
    #[getter]
    fn source(&mut self, py: Python<'_>) -> Option<pyo3::Py<MessageSource>> {
        match &self.source {
            Some(src) => Some(src.clone_ref(py)),
            None => {
                let src = MessageSource::from((*self.inner).clone());
                let py_src = Py::new(py, src).unwrap();
                self.source = Some(py_src.clone_ref(py));
                Some(py_src)
            }
        }
    }
}

#[pyclass]
pub enum UnavailableType {
    Unknown,
    ViewOnce,
}

#[pyclass]
pub enum DecryptFailMode {
    Show,
    Hide
}


#[pyclass]
pub struct EvUndecryptableMessage {
    info_inner: Box<wacore::types::message::MessageInfo>,
    info: Option<pyo3::Py<MessageInfo>>,
    #[pyo3(get)]
    is_unavailable: bool,
    #[pyo3(get)]
    unavailable_type: Py<UnavailableType>,
    #[pyo3(get)]
    decrypt_fail_mode: Py<DecryptFailMode>,
}

impl EvUndecryptableMessage {
    pub fn new(
        info_inner: wacore::types::message::MessageInfo,
        is_unavailable: bool,
        unavailable_type: wacore::types::events::UnavailableType,
        decrypt_fail_mode: wacore::types::events::DecryptFailMode,
    ) -> Self {
        let py_unavailable_type = match unavailable_type {
            wacore::types::events::UnavailableType::Unknown => UnavailableType::Unknown,
            wacore::types::events::UnavailableType::ViewOnce => UnavailableType::ViewOnce,
        };
        let py_decrypt_fail_mode = match decrypt_fail_mode {
            wacore::types::events::DecryptFailMode::Show => DecryptFailMode::Show,
            wacore::types::events::DecryptFailMode::Hide => DecryptFailMode::Hide,
        };

        Python::attach(|py| {
            Self {
                info_inner: Box::new(info_inner),
                info: None,
                is_unavailable,
                unavailable_type: Py::new(py, py_unavailable_type).unwrap(),
                decrypt_fail_mode: Py::new(py, py_decrypt_fail_mode).unwrap(),
            }
        })
    }
}

#[pymethods]
impl EvUndecryptableMessage {
    #[getter]
    fn info(&mut self, py: Python<'_>) -> Option<pyo3::Py<MessageInfo>> {
        match &self.info {
            Some(info) => Some(info.clone_ref(py)),
            None => {
                let info = MessageInfo::from((*self.info_inner).clone());
                let py_info = Py::new(py, info).unwrap();
                self.info = Some(py_info.clone_ref(py));
                Some(py_info)
            }
        }
    }
}

#[pyclass]
pub struct EvNotification{
    inner: wacore_binary::node::Node,
    node_cache: OnceLock<Py<Node>>,
}
impl EvNotification {
    pub fn new(inner: wacore_binary::node::Node) -> Self {
        Self { inner, node_cache: OnceLock::new() }
    }
}
#[pymethods]
impl EvNotification {
    #[getter]
    fn node(&mut self, py: Python<'_>) -> PyResult<Py<Node>> {
        if let Some(node) = self.node_cache.get() {
            Ok(node.clone_ref(py))
        } else {
            let node = Node::from_node(&self.inner);
            let py_node = Py::new(py, node)?;
            self.node_cache.set(py_node.clone_ref(py)).ok();
            Ok(py_node)
        }
    }
}

#[pyclass]
pub enum ChatPresence {
    Composing,
    Paused
}

#[pymethods]
impl ChatPresence {
    fn __str__(&self) -> &str {
        match self {
            ChatPresence::Composing => "composing",
            ChatPresence::Paused => "paused",
        }
    }
    fn __repr__(&self) -> &str {
        self.__str__()
    }
}

#[pyclass]
pub enum ChatPresenceMedia {
    Text,
    Audio
}

#[pymethods]
impl ChatPresenceMedia {
    fn __str__(&self) -> &str {
        match self {
            ChatPresenceMedia::Text => "text",
            ChatPresenceMedia::Audio => "audio",
        }
    }
    fn __repr__(&self) -> &str {
        self.__str__()
    }
}

#[pyclass]
pub struct EvChatPresence {
    source: Box<wacore::types::message::MessageSource>,
    source_cache: OnceLock<pyo3::Py<MessageSource>>,
    state: ChatPresence,
    media: ChatPresenceMedia
}

impl EvChatPresence {
    pub fn new(source: wacore::types::message::MessageSource, state: wacore::types::presence::ChatPresence, media: wacore::types::presence::ChatPresenceMedia) -> Self {
        let chat_presence_state = match state {
            wacore::types::presence::ChatPresence::Composing => ChatPresence::Composing,
            wacore::types::presence::ChatPresence::Paused => ChatPresence::Paused,
        };
        let chat_presence_media = match media {
            wacore::types::presence::ChatPresenceMedia::Text => ChatPresenceMedia::Text,
            wacore::types::presence::ChatPresenceMedia::Audio => ChatPresenceMedia::Audio,
        };
        Self {
            source: Box::new(source),
            source_cache: OnceLock::new(),
            state: chat_presence_state,
            media: chat_presence_media,
        }
    }
}

impl From<wacore::types::events::ChatPresenceUpdate> for EvChatPresence {
    fn from(event: wacore::types::events::ChatPresenceUpdate) -> Self {
        EvChatPresence::new(event.source, event.state, event.media)
    }
}

#[pymethods]
impl EvChatPresence {
    #[getter]
    fn source(&mut self, py: Python<'_>) -> Py<MessageSource> {
        if let Some(cached) = self.source_cache.get() {
            cached.clone_ref(py)
        } else {
            let py_source = Py::new(py, MessageSource::from((*self.source).clone())).unwrap();
            self.source_cache.set(py_source.clone_ref(py)).ok();
            py_source
        }
    }

    #[getter]
    fn state(&self) -> &'static str {
        match self.state {
            ChatPresence::Composing => "composing",
            ChatPresence::Paused => "paused",
        }
    }

    #[getter]
    fn media(&self) -> &'static str {
        match self.media {
            ChatPresenceMedia::Text => "text",
            ChatPresenceMedia::Audio => "audio",
        }
    }
}

#[pyclass]
pub struct EvPresence {
    #[pyo3(get)]
    from: Py<JID>,
    #[pyo3(get)]
    unavailable: bool,
    #[pyo3(get)]
    last_seen: Option<Py<PyDateTime>>,
}
impl EvPresence {
    pub fn new(from: Jid, unavailable: bool, last_seen: Option<DateTime<Utc>>) -> Self {
        Python::attach(|py|{
            let py_last_seen = last_seen.map(|dt| PyDateTime::from_timestamp(py, dt.timestamp() as f64, None).unwrap().into());
            Self { from: Py::new(py, JID::from(from)).unwrap(), unavailable, last_seen: py_last_seen }
        })
    }
}

impl From<wacore::types::events::PresenceUpdate> for EvPresence {
    fn from(event: wacore::types::events::PresenceUpdate) -> Self {
        EvPresence::new(event.from, event.unavailable, event.last_seen)
    }
}

#[pyclass]
pub struct PictureUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    author: Option<Py<JID>>,
    #[pyo3(get)]
    timestamp: Option<Py<PyDateTime>>,
    #[pyo3(get)]
    removed: bool,
    #[pyo3(get)]
    picture_id: Option<String>,
}
#[pyclass]
pub struct EvPictureUpdate{
    inner: wacore::types::events::PictureUpdate,
    data_cached: OnceLock<Py<PictureUpdateData>>,
}
impl EvPictureUpdate {
    pub fn new(inner: wacore::types::events::PictureUpdate) -> Self {
        Self { inner, data_cached: OnceLock::new() }
    }

}
impl From<wacore::types::events::PictureUpdate> for EvPictureUpdate {
    fn from(event: wacore::types::events::PictureUpdate) -> Self {
        EvPictureUpdate::new(event)
    }
}
#[pymethods]
impl EvPictureUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<PictureUpdateData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let new_data = PictureUpdateData {
                jid: Py::new(py, JID::from(self.inner.jid.clone())).unwrap(),
                author: self.inner.author.clone().map(|a| Py::new(py, JID::from(a)).unwrap()),
                timestamp: Some(PyDateTime::from_timestamp(py, self.inner.timestamp.timestamp() as f64, None).unwrap().unbind()),
                removed: self.inner.removed,
                picture_id: self.inner.picture_id.clone(),
            };
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}

#[pyclass]
pub struct UserAboutUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    timestamp: Option<Py<PyDateTime>>,
}

#[pyclass]
pub struct EvUserAboutUpdate{
    inner: wacore::types::events::UserAboutUpdate,
    data_cached: OnceLock<Py<UserAboutUpdateData>>,
}

impl EvUserAboutUpdate {
    pub fn new(inner: wacore::types::events::UserAboutUpdate) -> Self {
        Self { inner, data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::UserAboutUpdate> for EvUserAboutUpdate {
    fn from(event: wacore::types::events::UserAboutUpdate) -> Self {
        EvUserAboutUpdate::new(event)
    }
}
#[pymethods]
impl EvUserAboutUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<UserAboutUpdateData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let new_data = UserAboutUpdateData { jid: Py::new(py, JID::from(self.inner.jid.clone())).unwrap(), status: self.inner.status.clone(), timestamp: Some(PyDateTime::from_timestamp(py, self.inner.timestamp.timestamp() as f64, None).unwrap().unbind()) };
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}

#[pyclass]
pub struct LazyConversation {
    inner: Box<wacore::types::events::LazyConversation>,
    parsed: Option<Py<PyAny>>,
}
impl LazyConversation {
    pub fn new(inner: wacore::types::events::LazyConversation) -> Self {
        Self { inner: Box::new(inner), parsed: None }
    }
}
#[pymethods]
impl LazyConversation {
    #[getter]
    fn conversation(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        if let Some(ref parsed) = self.parsed {
            Ok(Some(parsed.clone_ref(py)))
        } else {
            let proto_type = get_lazy_conversation_proto_type(py)?;
            let proto = self.inner.get().ok_or_else(|| PyErr::new::<pyo3::exceptions::PyAttributeError, _>("LazyConversation does not contain conversation data"))?;
            let mut proto_bytes = Vec::with_capacity(proto.encoded_len());
            proto.encode(&mut proto_bytes).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to encode conversation proto: {}", e)))?;
            let parsed_proto = from_string_to_python_proto(py, proto_type, &proto_bytes)?;
            self.parsed = Some(parsed_proto.clone_ref(py));
            Ok(Some(parsed_proto))
        }
    }
}

#[pyclass]
pub struct EvJoinedGroup{
    inner: wacore::types::events::LazyConversation,
    conversation_cached: Option<Py<LazyConversation>>,
}
impl EvJoinedGroup {
    pub fn new(inner: wacore::types::events::LazyConversation) -> Self {
        Self { inner, conversation_cached: None }
    }
}
#[pymethods]
impl EvJoinedGroup {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<LazyConversation>> {
        if let Some(ref cached) = self.conversation_cached {
            Ok(cached.clone_ref(py))
        } else {
            let lazy_conversation = Py::new(py, LazyConversation::new(self.inner.clone())).unwrap();
            self.conversation_cached = Some(lazy_conversation.clone_ref(py));
            Ok(lazy_conversation)
        }
    }
}

#[pyclass]
pub struct EvGroupInfoUpdate;


include!("types/profile_sync.rs");
include!("types/message_and_updates.rs");
