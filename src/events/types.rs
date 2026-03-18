use std::sync::Arc;

use prost::Message as ProstMessage;
use pyo3::{Py, PyAny, PyErr, PyResult, Python,pyclass, pymethods, types::{PyAnyMethods, PyBytes, PyType}};
use pyo3::types::{PyDateTime};
use chrono::{DateTime, Utc};
use whatsapp_rust::types::events::{LoggedOut as WhatsAppLoggedOut, ConnectFailureReason };
use pyo3::sync::PyOnceLock;
use whatsapp_rust::types::message::{MessageInfo as WhatsappMessageInfo};
use crate::types::{JID, MessageInfo};

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
fn parse_sync_action_value_proto(py: Python<'_>, proto_bytes: &[u8]) -> PyResult<Py<PyAny>> {
    let proto_type = get_proto_sync_action_value_from_string(py)?;
    let proto_instance = proto_type.bind(py).call0()?;
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
    reason: &'static str,
}
impl EvLoggedOut {
    pub fn new(logout: WhatsAppLoggedOut) -> Self {
        Self { on_connect: logout.on_connect, reason: &connect_failure_reason_to_string(&logout.reason) }
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

#[pyclass]
pub struct EvQrScannedWithoutMultidevice;


#[pyclass]
pub struct EvClientOutDated;

#[pyclass]
pub struct EvReceipt;

#[pyclass]
pub struct EvUndecryptableMessage;

#[pyclass]
pub struct EvNotification;

#[pyclass]
pub struct EvChatPresence;

#[pyclass]
pub struct EvPresence;

#[pyclass]
pub struct EvPictureUpdate;

#[pyclass]
pub struct EvUserAboutUpdate;

#[pyclass]
pub struct EvJoinedGroup;

#[pyclass]
pub struct EvGroupInfoUpdate;

#[pyclass]
pub struct EvContactUpdate;

#[pyclass]
pub struct EvPushNameUpdate;

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

#[pyclass]
pub struct EvConnectFailure;

#[pyclass]
pub struct EvStreamError;

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

pub fn connect_failure_reason_to_string(reason: &ConnectFailureReason) -> &'static str {
    match reason {
        ConnectFailureReason::BadUserAgent => "BadUserAgent",
        ConnectFailureReason::LoggedOut => "LoggedOut",
        ConnectFailureReason::CatExpired => "CatExpired",
        ConnectFailureReason::CatInvalid => "CatInvalid",
        ConnectFailureReason::ClientOutdated => "ClientOutdated",
        ConnectFailureReason::ClientUnknown => "ClientUnknown",
        ConnectFailureReason::Generic => "Generic",
        ConnectFailureReason::TempBanned => "TempBanned",
        ConnectFailureReason::UnknownLogout => "Unknown",
        ConnectFailureReason::MainDeviceGone => "MainDeviceGone",
        ConnectFailureReason::NotFound => "NotFound",
        ConnectFailureReason::ServiceUnavailable => "ServiceUnavailable",
        ConnectFailureReason::InternalServerError => "InternalServerError",
        ConnectFailureReason::Experimental => "Experimental",
        ConnectFailureReason::Unknown(_) => "Unknown",
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