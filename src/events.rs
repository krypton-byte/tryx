use std::sync::Arc;

use prost::Message as ProstMessage;
use pyo3::{Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods, types::{PyAnyMethods, PyBytes, PyType}};
use whatsapp_rust::types::events::{LoggedOut as WhatsAppLoggedOut, ConnectFailureReason };
use pyo3::sync::PyOnceLock;
use whatsapp_rust::types::message::{MessageInfo as WhatsappMessageInfo};
use crate::types::{JID, MessageInfo};

static WHATSAPP_MESSAGE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();

fn get_proto_message_from_string(py: Python<'_>) -> Result<&Py<PyType>, PyErr> {
    let proto_type = WHATSAPP_MESSAGE_PROTO.get_or_try_init(py, || -> PyResult<Py<PyType>> {
        let proto_module = py.import("tryx.waproto.whatsapp_pb2")?;
        let message_type = proto_module.getattr("Message")?.cast_into::<PyType>()?;
        Ok(message_type.unbind())
    })?;
    Ok(proto_type)
}

fn parse_message_proto(py: Python<'_>, proto_bytes: &[u8]) -> PyResult<Py<PyAny>> {
    let proto_type = get_proto_message_from_string(py)?;
    let proto_instance = proto_type.bind(py).call0()?;
    proto_instance.call_method1("ParseFromString", (PyBytes::new(py, proto_bytes),))?;
    Ok(proto_instance.into_any().unbind())
}


#[pyclass]
pub struct Connected;

#[pyclass]
pub struct Disconnected;


#[pyclass]
pub struct LoggedOut{
    #[pyo3(get)]
    on_connect: bool,
    #[pyo3(get)]
    reason: &'static str,
}
impl LoggedOut {
    pub fn new(logout: WhatsAppLoggedOut) -> Self {
        Self { on_connect: logout.on_connect, reason: &connect_failure_reason_to_string(&logout.reason) }
    }
}

#[pyclass]
pub struct PairSuccess {
    id: JID,
    lid: JID,
    #[pyo3(get)]
    business_name: String,
    #[pyo3(get)]
    platform: String,
}

#[pymethods]
impl PairSuccess {
    #[getter]
    fn id(&self) -> JID {
        self.id.clone()
    }
    #[getter]
    fn lid(&self) -> JID {
        self.lid.clone()
    }
}

#[pyclass]
pub struct PairError {
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
pub struct PairingQrCode {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    timeout: u64,
}
impl PairingQrCode {
    pub fn new(code: String, timeout: u64) -> Self {
        Self { code, timeout }
    }

}

#[pyclass]
pub struct PairingCode {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    timeout: u64,
}

#[pyclass]
pub struct QrScannedWithoutMultidevice;


#[pyclass]
pub struct ClientOutDated;

#[pyclass]
pub struct Message {
    inner: Box<waproto::whatsapp::Message>,
    #[pyo3(get)]
    message_info: MessageInfo,
    message_proto: Option<Py<PyAny>>,
}
impl Message {
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
impl Message {
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