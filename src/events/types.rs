use std::sync::Arc;

use prost::Message as ProstMessage;
use pyo3::{Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3::types::{PyAnyMethods, PyBytes, PyType};
use pyo3::types::{PyDateTime};
use chrono::{DateTime, Utc};
use wacore::types::events::QrScannedWithoutMultidevice;
use whatsapp_rust::{Jid, types::events::{ConnectFailureReason, LoggedOut as WhatsAppLoggedOut }};
use pyo3::sync::PyOnceLock;
use whatsapp_rust::types::message::{MessageInfo as WhatsappMessageInfo};
use crate::types::{JID, MessageInfo, MessageSource};
use crate::wacore::node::Node;
static WHATSAPP_MESSAGE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static SYNC_ACTION_VALUE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
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
    node_cache: Option<Py<Node>>,
}
impl EvNotification {
    pub fn new(inner: wacore_binary::node::Node) -> Self {
        Self { inner, node_cache: None }
    }
}
#[pymethods]
impl EvNotification {
    #[getter]
    fn node(&mut self, py: Python<'_>) -> PyResult<Py<Node>> {
        if let Some(ref node) = self.node_cache {
            Ok(node.clone_ref(py))
        } else {
            let py_node = Py::new(py, Node::from_node(&self.inner)).unwrap();
            self.node_cache = Some(py_node.clone_ref(py));
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
    source_cache: Option<pyo3::Py<MessageSource>>,
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
        Self { source, source_cache: None, state: chat_presence_state, media: chat_presence_media }
    }
}

impl From<wacore::types::events::ChatPresenceUpdate> for EvChatPresence {
    fn from(event: wacore::types::events::ChatPresenceUpdate) -> Self {
        EvChatPresence::new(Arc::new(event.source), event.state, event.media)
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
    data_cached: Option<Py<PictureUpdateData>>,
}
impl EvPictureUpdate {
    pub fn new(inner: wacore::types::events::PictureUpdate) -> Self {
        Self { inner, data_cached: None }
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
        if let Some(ref data) = self.data_cached {
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
            self.data_cached = Some(py_data.clone_ref(py));
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
    data_cached: Option<Py<UserAboutUpdateData>>,
}

impl EvUserAboutUpdate {
    pub fn new(inner: wacore::types::events::UserAboutUpdate) -> Self {
        Self { inner, data_cached: None }
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
        if let Some(ref data) = self.data_cached {
            data.clone_ref(py)
        } else {
            let new_data = UserAboutUpdateData { jid: self.inner.jid.into(), status: self.inner.status.clone(), timestamp: self.inner.timestamp.map(|dt| Python::attach(|py| PyDateTime::from_timestamp(py, dt.timestamp() as f64, None).unwrap().into())) };
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached = Some(py_data.clone_ref(py));
            py_data
        }
    }
}

#[pyclass]
pub struct JoinedGroupData {
    #[pyo3(get)]
    chat: JID,
    #[pyo3(get)]
    sender: JID,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
}

#[pyclass]
pub struct EvJoinedGroup{

}

#[pyclass]
pub struct EvGroupInfoUpdate;

#[pyclass]
pub struct EvContactUpdate;

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
pub struct EvSelfPushNameUpdated;

#[pyclass]
pub struct EvPinUpdate;

#[pyclass]
pub struct EvMuteUpdate;

#[pyclass]
pub struct EvMarkChatAsReadUpdate;

#[pyclass]
pub struct EvHistorySync;

#[pyclass]
pub struct EvOfflineSyncPreview;

#[pyclass]
pub struct EvOfflineSyncCompleted;

#[pyclass]
pub struct EvDeviceListUpdate;

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
