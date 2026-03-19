use std::sync::{Arc, Once};

use prost::Message as ProstMessage;
use pyo3::{Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3::types::{PyAnyMethods, PyBytes, PyType};
use pyo3::types::{PyDateTime};
use chrono::{DateTime, Utc};
use std::sync::OnceLock;
use wacore::types::events::QrScannedWithoutMultidevice;
use whatsapp_rust::{Jid, types::events::{ConnectFailureReason, LoggedOut as WhatsAppLoggedOut }};
use pyo3::sync::PyOnceLock;
use whatsapp_rust::types::message::{MessageInfo as WhatsappMessageInfo};
use crate::types::{JID, MessageInfo, MessageSource};
use crate::wacore::node::Node;
use crate::wacore::stanza::KeyIndexInfo;
static WHATSAPP_MESSAGE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static SYNC_ACTION_VALUE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static LAZY_CONVERSATION_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static CONTACT_UPDATE_ACTION_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static MARK_CHAT_AS_READ_UPDATE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static HISTORY_SYNC_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
fn get_proto_import(py: Python<'_>, import: &str, attr: &str) -> PyResult<Py<PyType>>{
    let module = py.import(import)?;
    let message_type = module.getattr(attr)?.cast_into::<PyType>()?;
    Ok(message_type.unbind())
}

fn get_proto_message_from_string(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let proto_type = WHATSAPP_MESSAGE_PROTO.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        get_proto_import(py, "tryx.waproto.whatsapp_pb2", "Message")
    })?;
    Ok(proto_type)
}

fn get_proto_sync_action_value_from_string(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let proto_type = SYNC_ACTION_VALUE.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        get_proto_import(py, "tryx.waproto.whatsapp_pb2", "SyncActionValue")
    })?;
    Ok(proto_type)
}
fn get_lazy_conversation_proto_type(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let conversation = LAZY_CONVERSATION_PROTO.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        get_proto_import(py, "tryx.waproto.whatsapp_pb2", "Conversation")
    })?;
    Ok(conversation)
}
fn get_proto_contact_action_proto_type(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let contact_action = CONTACT_UPDATE_ACTION_PROTO.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        get_proto_import(py, "tryx.waproto.whatsapp_pb2", "ContactAction")
    })?;
    Ok(contact_action)
}
fn get_proto_mute_action_proto_type(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let mute_action = SYNC_ACTION_VALUE.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        get_proto_import(py, "tryx.waproto.whatsapp_pb2", "MuteAction")
    })?;
    Ok(mute_action)
}
fn get_proto_mark_chat_as_read_action_proto_type(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let mark_chat_as_read_action = MARK_CHAT_AS_READ_UPDATE_PROTO.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        get_proto_import(py, "tryx.waproto.whatsapp_pb2", "MarkChatAsReadAction")
    })?;
    Ok(mark_chat_as_read_action)
}
fn get_proto_history_sync_proto_type(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let history_sync = HISTORY_SYNC_PROTO.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        get_proto_import(py, "tryx.waproto.whatsapp_pb2", "HistorySync")
    })?;
    Ok(history_sync)
}
fn from_string_to_python_proto(py: Python<'_>, proto_class: &Py<PyType>, proto_bytes: &[u8]) -> PyResult<Py<PyAny>> {
    let proto_instance = proto_class.bind(py).call0()?;
    proto_instance.call_method1("ParseFromString", (PyBytes::new(py, proto_bytes),))?;
    Ok(proto_instance.into_any().unbind())
}
fn parse_message_proto(py: Python<'_>, proto_bytes: &[u8]) -> PyResult<Py<PyAny>> {
    let proto_type = get_proto_message_from_string(py)?;
    let proto_instance = proto_type.bind(py).call0()?;
    proto_instance.call_method1("ParseFromString", (PyBytes::new(py, proto_bytes),))?;
    Ok(proto_instance.into_any().unbind())
}



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
pub struct EvPairSuccess {
    #[pyo3(get)]
    id: JID,
    #[pyo3(get)]
    lid: JID,
    #[pyo3(get)]
    business_name: String,
    #[pyo3(get)]
    platform: String,
}
impl EvPairSuccess {
    pub fn new(id: JID, lid: JID, business_name: String, platform: String) -> Self {
        Self { id, lid, business_name, platform }
    }
}

#[pyclass]
pub struct EvPairError {
    #[pyo3(get)]
    id: JID,
    #[pyo3(get)]
    lid: JID,
    #[pyo3(get)]
    business_name: String,
    #[pyo3(get)]
    platform: String,
    #[pyo3(get)]
    error: String,
}
impl EvPairError {
    pub fn new(id: JID, lid: JID, business_name: String, platform: String, error: String) -> Self {
        Self { id, lid, business_name, platform, error }
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
enum ReceiptType {
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
    inner: Arc<wacore::types::message::MessageSource>,
    source: Option<pyo3::Py<MessageSource>>,
    #[pyo3(get)]
    message_ids: Vec<String>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    receipt_type: Py<ReceiptType>,
    #[pyo3(get)]
    message_sender: JID
}

impl EvReceipt {
    pub fn new(inner: Arc<wacore::types::message::MessageSource>, message_ids: Vec<String>, timestamp: DateTime<Utc>, r#type: wacore::types::presence::ReceiptType,message_sender: Jid)-> Py<Self> {
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
            wacore::types::presence::ReceiptType::Other(_) => ReceiptType::Other
            
        };
        Python::attach(|py|{
            pyo3::Py::new(py, Self{
                inner,
                source: None,
                message_ids,
                timestamp: PyDateTime::from_timestamp(py, timestamp.naive_utc().and_utc().timestamp_millis() as f64 / 1000.0, None).unwrap().into(),
                receipt_type: Py::new(py, receipt_type).unwrap(),
                message_sender: message_sender.into(),
            })
        }).unwrap()
    }
}

#[pymethods]
impl EvReceipt {
    #[getter]
    fn source(&mut self) -> Option<pyo3::Py<MessageSource>> {
        Python::attach(|py|{
            match &self.source {
                Some(src) => Some(src.clone_ref(py)),
                None => {
                    let src = MessageSource::from((*self.inner).clone());
                    let py_src = Py::new(py, src).unwrap();
                    self.source = Some(py_src.clone_ref(py));
                    Some(py_src)
                }
            }
        })
    }
}

#[pyclass]
enum UnavailableType {
    Unknown,
    ViewOnce,
}

#[pyclass]
enum DecryptFailMode {
    Show,
    Hide
}


#[pyclass]
pub struct EvUndecryptableMessage {
    info_inner: Arc<wacore::types::message::MessageInfo>,
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
        info_inner: Arc<wacore::types::message::MessageInfo>,
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

        Self {
            info_inner,
            info: None,
            is_unavailable,
            unavailable_type: Python::attach(|py| Py::new(py, py_unavailable_type)).unwrap(),
            decrypt_fail_mode: Python::attach(|py| Py::new(py, py_decrypt_fail_mode)).unwrap(),
        }
    }
}

#[pymethods]
impl EvUndecryptableMessage {
    #[getter]
    fn info(&mut self) -> Option<pyo3::Py<MessageInfo>> {
        Python::attach(|py|{
            match &self.info {
                Some(info) => Some(info.clone_ref(py)),
                None => {
                    let info = MessageInfo::from((*self.info_inner).clone());
                    let py_info = Py::new(py, info).unwrap();
                    self.info = Some(py_info.clone_ref(py));
                    Some(py_info)
                }
            }
        })
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
enum ChatPresence {
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
enum ChatPresenceMedia {
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
    source: Arc<wacore::types::message::MessageSource>,
    source_cache: OnceLock<pyo3::Py<MessageSource>>,
    state: ChatPresence,
    media: ChatPresenceMedia
}

impl EvChatPresence {
    pub fn new(source: Arc<wacore::types::message::MessageSource>, state: wacore::types::presence::ChatPresence, media: wacore::types::presence::ChatPresenceMedia) -> Self {
        let chat_presence_state = match state {
            wacore::types::presence::ChatPresence::Composing => ChatPresence::Composing,
            wacore::types::presence::ChatPresence::Paused => ChatPresence::Paused,
        };
        let chat_presence_media = match media {
            wacore::types::presence::ChatPresenceMedia::Text => ChatPresenceMedia::Text,
            wacore::types::presence::ChatPresenceMedia::Audio => ChatPresenceMedia::Audio,
        };
        Self { source, source_cache: OnceLock::new(), state: chat_presence_state, media: chat_presence_media }
    }
}

impl From<wacore::types::events::ChatPresenceUpdate> for EvChatPresence {
    fn from(event: wacore::types::events::ChatPresenceUpdate) -> Self {
        EvChatPresence::new(Arc::new(event.source), event.state, event.media)
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
    from: JID,
    #[pyo3(get)]
    unavailable: bool,
    #[pyo3(get)]
    last_seen: Option<Py<PyDateTime>>,
}
impl EvPresence {
    pub fn new(from: Jid, unavailable: bool, last_seen: Option<DateTime<Utc>>) -> Self {
        let py_last_seen = last_seen.map(|dt| Python::attach(|py| PyDateTime::from_timestamp(py, dt.timestamp() as f64, None).unwrap().into()));
        Self { from: from.into(), unavailable, last_seen: py_last_seen }
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
    jid: JID,
    #[pyo3(get)]
    author: Option<JID>,
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
                jid: self.inner.jid.clone().into(),
                author: self.inner.author.clone().map(|a| a.into()),
                timestamp: Some(Python::attach(|py| {
                        PyDateTime::from_timestamp(py, self.inner.timestamp.timestamp() as f64, None).unwrap().unbind()
                    })),
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
    jid: JID,
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
            let new_data = UserAboutUpdateData { jid: self.inner.jid.clone().into(), status: self.inner.status.clone(), timestamp: Some(Python::attach(|py| PyDateTime::from_timestamp(py, self.inner.timestamp.timestamp() as f64, None).unwrap().unbind())) };
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}

#[pyclass]
pub struct LazyConversation {
    inner: Arc<wacore::types::events::LazyConversation>,
    parsed: Option<Py<PyAny>>,
}
impl LazyConversation {
    pub fn new(inner: Arc<wacore::types::events::LazyConversation>) -> Self {
        Self { inner, parsed: None }
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
            let mut proto_bytes = Vec::new();
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
            let lazy_conversation = Py::new(py, LazyConversation::new(Arc::new(self.inner.clone()))).unwrap();
            self.conversation_cached = Some(lazy_conversation.clone_ref(py));
            Ok(lazy_conversation)
        }
    }
}

#[pyclass]
pub struct EvGroupInfoUpdate;

#[pyclass]
pub struct EvPushNameUpdate {
    #[pyo3(get)]
    jid: JID,
    #[pyo3(get)]
    message: Py<MessageInfo>,
    #[pyo3(get)]
    old_push_name: String,
    #[pyo3(get)]
    new_push_name: String,
}
impl EvPushNameUpdate {
    pub fn new(jid: JID, message: MessageInfo, old_push_name: String, new_push_name: String) -> Self {
        Self { jid, message: Python::attach(|py| Py::new(py, message).unwrap()), old_push_name, new_push_name }
    }
}


#[pyclass]
pub struct EvSelfPushNameUpdated {
    #[pyo3(get)]
    from_server: bool,
    #[pyo3(get)]
    old_name: String,
    #[pyo3(get)]
    new_name: String,
}
impl From<wacore::types::events::SelfPushNameUpdated> for EvSelfPushNameUpdated {
    fn from(event: wacore::types::events::SelfPushNameUpdated) -> Self {
        EvSelfPushNameUpdated { from_server: event.from_server, old_name: event.old_name, new_name: event.new_name }
    }
}

#[pyclass]
pub struct PinUpdatedata {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pinned: Option<bool>,
    #[pyo3(get)]
    from_full_sync: bool,
}
impl PinUpdatedata {
    pub fn new(jid: Jid, timestamp: DateTime<Utc>, action: waproto::whatsapp::sync_action_value::PinAction, from_full_sync: bool) -> Self {
        Self { jid: Python::attach(|py| Py::new(py, JID::from(jid)).unwrap()), timestamp: Python::attach(|py| PyDateTime::from_timestamp(py, timestamp.timestamp() as f64, None).unwrap().into()), pinned: action.pinned, from_full_sync }
    }
}

#[pyclass]
pub struct EvPinUpdate {
    inner: Arc<wacore::types::events::PinUpdate>,
    data_cached: OnceLock<Py<PinUpdatedata>>,
}
impl EvPinUpdate {
    pub fn new(inner: wacore::types::events::PinUpdate) -> Self {
        Self { inner: Arc::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::PinUpdate> for EvPinUpdate {
    fn from(event: wacore::types::events::PinUpdate) -> Self {
        EvPinUpdate::new(event)
    }
}
#[pymethods]
impl EvPinUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<PinUpdatedata>> {
        if let Some(ref data) = self.data_cached.get() {
            Ok(data.clone_ref(py))
        } else {
            let new_data = PinUpdatedata::new(self.inner.jid.clone(), self.inner.timestamp, (*self.inner.action).clone(), self.inner.from_full_sync);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}



#[pyclass]
pub struct MuteUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    action: Arc<waproto::whatsapp::sync_action_value::MuteAction>,
    action_cached: OnceLock<Py<PyAny>>,
    #[pyo3(get)]
    from_full_sync: bool,
}

impl MuteUpdateData {
    pub fn new(jid: JID, timestamp: DateTime<Utc>, action: waproto::whatsapp::sync_action_value::MuteAction, from_full_sync: bool) -> Self {
        Self { jid: Python::attach(|py| Py::new(py, JID::from(jid)).unwrap()), timestamp: Python::attach(|py| PyDateTime::from_timestamp(py, timestamp.timestamp() as f64, None).unwrap().into()), action: Arc::new(action), action_cached: OnceLock::new(), from_full_sync }
    }
}

#[pymethods]
impl MuteUpdateData {
    #[getter]
    fn action(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(cached) = self.action_cached.get() {
            Ok(cached.clone_ref(py))
        } else {
            let proto_type = get_proto_mute_action_proto_type(py)?;
            let mut proto_bytes = Vec::new();
            self.action.encode(&mut proto_bytes).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to encode MuteAction proto: {}", e)))?;
            let parsed_proto = from_string_to_python_proto(py, proto_type, &proto_bytes)?;
            self.action_cached.set(parsed_proto.clone_ref(py)).ok();
            Ok(parsed_proto)
        }
    }
}

#[pyclass]
pub struct EvMuteUpdate {
    inner: Arc<wacore::types::events::MuteUpdate>,
    data_cached: OnceLock<Py<MuteUpdateData>>,
}
impl EvMuteUpdate {
    pub fn new(inner: wacore::types::events::MuteUpdate) -> Self {
        Self { inner: Arc::new(inner), data_cached: OnceLock::new() }
    }
}
#[pymethods]
impl EvMuteUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<MuteUpdateData>> {
        if let Some(ref data) = self.data_cached.get() {
            Ok(data.clone_ref(py))
        } else {
            let new_data = MuteUpdateData::new(self.inner.jid.clone().into(), self.inner.timestamp, (*self.inner.action).clone(), self.inner.from_full_sync);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}
impl From<wacore::types::events::MuteUpdate> for EvMuteUpdate {
    fn from(event: wacore::types::events::MuteUpdate) -> Self {
        EvMuteUpdate::new(event)
    }
}

#[pyclass]
pub struct MarkChatAsReadUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    action: Arc<waproto::whatsapp::sync_action_value::MarkChatAsReadAction>,
    action_cached: OnceLock<Py<PyAny>>,
    #[pyo3(get)]
    from_full_sync: bool,
}
impl MarkChatAsReadUpdateData {
    pub fn new(jid: JID, timestamp: DateTime<Utc>, action: waproto::whatsapp::sync_action_value::MarkChatAsReadAction, from_full_sync: bool) -> Self {
        Self { jid: Python::attach(|py| Py::new(py, JID::from(jid)).unwrap()), timestamp: Python::attach(|py| PyDateTime::from_timestamp(py, timestamp.timestamp() as f64, None).unwrap().into()), action: Arc::new(action), action_cached: OnceLock::new(), from_full_sync }
    }
}
#[pymethods]
impl MarkChatAsReadUpdateData {
    #[getter]
    fn action(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(cached) = self.action_cached.get() {
            Ok(cached.clone_ref(py))
        } else {
            let proto_type = get_proto_mark_chat_as_read_action_proto_type(py)?;
            let mut proto_bytes = Vec::new();
            self.action.encode(&mut proto_bytes).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to encode MarkChatAsReadAction proto: {}", e)))?;
            let parsed_proto = from_string_to_python_proto(py, proto_type, &proto_bytes)?;
            self.action_cached.set(parsed_proto.clone_ref(py)).ok();
            Ok(parsed_proto)
        }
    }
}
#[pyclass]
pub struct EvMarkChatAsReadUpdate {
    inner: Arc<wacore::types::events::MarkChatAsReadUpdate>,
    data_cached: OnceLock<Py<MarkChatAsReadUpdateData>>,
}
impl EvMarkChatAsReadUpdate {
    pub fn new(inner: wacore::types::events::MarkChatAsReadUpdate) -> Self {
        Self { inner: Arc::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::MarkChatAsReadUpdate> for EvMarkChatAsReadUpdate {
    fn from(event: wacore::types::events::MarkChatAsReadUpdate) -> Self {
        EvMarkChatAsReadUpdate::new(event)
    }
}
#[pymethods]
impl EvMarkChatAsReadUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<MarkChatAsReadUpdateData>> {
        if let Some(ref data) = self.data_cached.get() {
            Ok(data.clone_ref(py))
        } else {
            let new_data = MarkChatAsReadUpdateData::new(JID::from(self.inner.jid.clone()), self.inner.timestamp, (*self.inner.action).clone(), self.inner.from_full_sync);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}



#[pyclass]
pub struct EvHistorySync{
    inner: Arc<waproto::whatsapp::HistorySync>,
    proto_cached: OnceLock<Py<PyAny>>,
}
impl EvHistorySync {
    pub fn new(inner: Arc<waproto::whatsapp::HistorySync>) -> Self {
        Self { inner, proto_cached: OnceLock::new() }
    }
}
impl From<waproto::whatsapp::HistorySync> for EvHistorySync {
    fn from(event: waproto::whatsapp::HistorySync) -> Self {
        EvHistorySync::new(Arc::new(event))
    }
}
#[pymethods]
impl EvHistorySync {
    #[getter]
    fn proto(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(ref proto) = self.proto_cached.get() {
            Ok(proto.clone_ref(py))
        } else {
            let proto_type = get_proto_history_sync_proto_type(py)?;
            let mut proto_bytes = Vec::new();
            self.inner.encode(&mut proto_bytes).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to encode HistorySync proto: {}", e)))?;
            let parsed_proto = from_string_to_python_proto(py, proto_type, &proto_bytes)?;
            self.proto_cached.set(parsed_proto.clone_ref(py)).ok();
            Ok(parsed_proto)
        }
    }
}
#[pyclass]
pub struct OfflineSyncData {
    #[pyo3(get)]
    total: i32,
    #[pyo3(get)]
    app_data_changes: i32,
    #[pyo3(get)]
    messages: i32,
    #[pyo3(get)]
    notifications: i32,
    #[pyo3(get)]
    receipts: i32,
}
impl OfflineSyncData {
    pub fn new(total: i32, app_data_changes: i32, messages: i32, notifications: i32, receipts: i32) -> Self {
        Self { total, app_data_changes, messages, notifications, receipts }
    }
}
#[pyclass]
pub struct EvOfflineSyncPreview {
    inner: Arc<wacore::types::events::OfflineSyncPreview>,
    data_cached: OnceLock<Py<OfflineSyncData>>,
}
#[pymethods]
impl EvOfflineSyncPreview {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<OfflineSyncData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let new_data = OfflineSyncData::new(self.inner.total, self.inner.app_data_changes, self.inner.messages, self.inner.notifications, self.inner.receipts);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}
impl EvOfflineSyncPreview {
    pub fn new(inner: Arc<wacore::types::events::OfflineSyncPreview>) -> Self {
        Self { inner, data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::OfflineSyncPreview> for EvOfflineSyncPreview {
    fn from(event: wacore::types::events::OfflineSyncPreview) -> Self {
        EvOfflineSyncPreview::new(Arc::new(event))
    }
}
#[pyclass]
pub struct OfflineSyncCompletedData {
    #[pyo3(get)]
    count: i32,
}
impl OfflineSyncCompletedData {
    pub fn new(count: i32) -> Self {
        Self { count }
    }
}

#[pyclass]
pub struct EvOfflineSyncCompleted{
    inner: Arc<wacore::types::events::OfflineSyncCompleted>,
    data_cached: OnceLock<Py<OfflineSyncCompletedData>>,
}

impl EvOfflineSyncCompleted {
    pub fn new(inner: Arc<wacore::types::events::OfflineSyncCompleted>) -> Self {
        Self { inner, data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::OfflineSyncCompleted> for EvOfflineSyncCompleted {
    fn from(event: wacore::types::events::OfflineSyncCompleted) -> Self {
        EvOfflineSyncCompleted::new(Arc::new(event))
    }
}
#[pymethods]
impl EvOfflineSyncCompleted {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<OfflineSyncCompletedData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let new_data = OfflineSyncCompletedData::new(self.inner.count);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}

#[pyclass]
enum DeviceListUpdateType {
    Added,
    Removed,
    Updated
}
#[pyclass]
struct DeviceNottificationInfo {
    #[pyo3(get)]
    device_id: u32,
    #[pyo3(get)]
    key_index: Option<u32>,
}
#[pyclass]
pub struct DeviceListUpdateData {
    #[pyo3(get)]
    user: JID,
    #[pyo3(get)]
    lid_user: Option<JID>,
    #[pyo3(get)]
    update_type: Py<DeviceListUpdateType>,
    #[pyo3(get)]
    devices: Vec<Py<DeviceNottificationInfo>>,
    #[pyo3(get)]
    key_index: Option<Py<KeyIndexInfo>>,
    #[pyo3(get)]
    contact_hash: Option<String>,
}

#[pyclass]
pub struct EvDeviceListUpdate {
    inner: Arc<wacore::types::events::DeviceListUpdate>,
    data_cached: OnceLock<Py<DeviceListUpdateData>>,
}
impl EvDeviceListUpdate {
    pub fn new(inner: Arc<wacore::types::events::DeviceListUpdate>) -> Self {
        Self { inner, data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::DeviceListUpdate> for EvDeviceListUpdate {
    fn from(event: wacore::types::events::DeviceListUpdate) -> Self {
        EvDeviceListUpdate::new(Arc::new(event))
    }
}
#[pymethods]
impl EvDeviceListUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<DeviceListUpdateData> {
        if let Some(data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let update_type = match self.inner.update_type {
                wacore::types::events::DeviceListUpdateType::Add => DeviceListUpdateType::Added,
                wacore::types::events::DeviceListUpdateType::Remove => DeviceListUpdateType::Removed,
                wacore::types::events::DeviceListUpdateType::Update => DeviceListUpdateType::Updated,
            };
            let devices = self.inner.devices.iter().map(|d| {
                Py::new(py, DeviceNottificationInfo { device_id: d.device_id, key_index: d.key_index }).unwrap()
            }).collect();
            let key_index = self.inner.key_index.clone().map(|k| Py::new(py, KeyIndexInfo::new(k.timestamp, k.signed_bytes)).unwrap());
            let new_data = DeviceListUpdateData {
                user: self.inner.user.clone().into(),
                lid_user: self.inner.lid_user.clone().map(|u| u.into()),
                update_type: Python::attach(|py| Py::new(py, update_type).unwrap()),
                devices,
                key_index,
                contact_hash: self.inner.contact_hash.clone(),
            };
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}

#[pyclass]
pub struct EvBusinessStatusUpdate;

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
pub struct EvMessage {
    pub inner: Box<waproto::whatsapp::Message>,
    #[pyo3(get)]
    pub message_info: MessageInfo,
    message_proto: Option<Py<PyAny>>,
}
impl EvMessage {
    pub fn new(inner: Box<waproto::whatsapp::Message>, message_info: WhatsappMessageInfo) -> Self {
        let info = MessageInfo {
            inner: Arc::new(message_info.clone()),
            id: message_info.id.clone(),
            r#type: message_info.r#type.clone(),
            push_name: message_info.push_name.clone(),
        };
        Self { inner, message_info: info, message_proto: None }
    }
}
#[pymethods]
impl EvMessage {
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
        match self.message_proto {
            Some(ref proto) => Ok(proto.clone_ref(py)),
            None => {
                let mut buffer = Vec::new();
                self.inner
                    .as_ref()
                    .encode(&mut buffer)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

                let new_proto = parse_message_proto(py, &buffer)?;
                let out_proto = new_proto.clone_ref(py);
                self.message_proto = Some(new_proto);
                Ok(out_proto.clone_ref(py))
            },
        }
    }
    fn __repr__(&self) -> String {
        format!("Message(id='{}', type='{}', push_name='{}')", self.message_info.id, self.message_info.r#type, self.message_info.push_name)
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
    #[pyo3(get)]
    jid: JID,
    timestamp: DateTime<Utc>,
    action: Arc<waproto::whatsapp::sync_action_value::ArchiveChatAction>,
    #[pyo3(get)]
    from_full_sync: bool,
    action_cache: Option<Py<PyAny>>,
}
impl EvArchiveUpdate {
    pub fn new(jid: JID, timestamp: DateTime<Utc>, action: Arc<waproto::whatsapp::sync_action_value::ArchiveChatAction>, from_full_sync: bool) -> Self {
        Self {
            jid,
            timestamp,
            action,
            from_full_sync,
            action_cache: None,
        }
    }
}
#[pymethods]
impl EvArchiveUpdate {
    #[getter]
    fn timestamp(&self, py: Python<'_>) -> PyResult<pyo3::Py<PyDateTime>> {
        let dt = self.timestamp.naive_utc();
        let py_dt = PyDateTime::from_timestamp(py, dt.and_utc().timestamp_millis() as f64 /1000.0, None).map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to convert timestamp: {}", e)))?;
        Ok(py_dt.into())
    }
    #[getter]
    fn action(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(ref cache) = self.action_cache {
            Ok(cache.clone_ref(py))
        } else {
            let proto_instance = from_string_to_python_proto(py, get_proto_sync_action_value_from_string(py)?, self.action.as_ref().encode_to_vec().as_slice())?;
            self.action_cache = Some(proto_instance.clone_ref(py));
            Ok(proto_instance)
        }

    }

}

#[pyclass]
pub struct EvDisappearingModeChanged {
    #[pyo3(get)]
    from: JID,
    #[pyo3(get)]
    duration: u32,
    #[pyo3(get)]
    setting_timestamp: u64
}

impl EvDisappearingModeChanged {
    pub fn new(from: Jid, duration: u32, setting_timestamp: u64) -> Self {
        Self { from: from.into(), duration, setting_timestamp }
    }
}

#[pyclass]
pub struct EvContactNumberChanged {
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

impl EvContactNumberChanged {
    pub fn new(old_jid: Jid, new_jid: Jid, old_lid: Option<Jid>, new_lid: Option<Jid>, timestamp: DateTime<Utc>) -> Self {
        let timestamp = Python::attach(|py| {
            PyDateTime::from_timestamp(
                py,
                (timestamp.timestamp_millis() as f64) / 1000.0,
                None
            ).unwrap().into()
        });
        Self {
            old_jid: old_jid.into(),
            new_jid: new_jid.into(),
            old_lid: old_lid.map(|j| j.into()),
            new_lid: new_lid.map(|j| j.into()),
            timestamp,
        }
    }
}

#[pyclass]
pub struct EvContactSyncRequested{
    #[pyo3(get)]
    after: Option<Py<PyDateTime>>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
}
impl EvContactSyncRequested {
    pub fn new(after: Option<DateTime<Utc>>, timestamp: DateTime<Utc>) -> Self {
        Python::attach(|py| {
            let after = after.map(|dt| PyDateTime::from_timestamp(py, (dt.timestamp_millis() as f64) / 1000.0, None).unwrap().into());
            let timestamp = PyDateTime::from_timestamp(py, (timestamp.timestamp_millis() as f64) / 1000.0, None).unwrap().into();
            Self { after, timestamp }
        })
    }
}

#[pyclass]
pub struct EvContactUpdated {
    #[pyo3(get)]
    jid: JID,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
}

impl EvContactUpdated {
    pub fn new(jid: Jid, timestamp: DateTime<Utc>) -> Self {
        let timestamp = Python::attach(|py| PyDateTime::from_timestamp(py, (timestamp.timestamp_millis() as f64) / 1000.0, None).unwrap().into());
        Self { jid: jid.into(), timestamp }
    }
}

#[pyclass]
pub struct EvStarUpdate {
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

impl EvStarUpdate {
    pub fn new(chat_jid: Jid, participant_jid: Option<Jid>, message_id: String, from_me: bool, timestamp: DateTime<Utc>, from_full_sync: bool, starred: Option<bool>) -> Self {
        let timestamp = Python::attach(|py| PyDateTime::from_timestamp(py, (timestamp.timestamp_millis() as f64) / 1000.0, None).unwrap().into());
        Self { chat_jid: chat_jid.into(), participant_jid: participant_jid.map(|j| j.into()), message_id, from_me, timestamp, from_full_sync, starred }
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
    action: Arc<waproto::whatsapp::sync_action_value::ContactAction>,
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
    inner: Arc<wacore::types::events::ContactUpdate>,
    contact_cache: OnceLock<Py<ContactUpdateData>>,

}   

impl EvContactUpdate {
    pub fn new(inner: wacore::types::events::ContactUpdate) -> Self {
        Self {
            inner:Arc::new(inner),
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
                action: Arc::from(self.inner.action.clone()),
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