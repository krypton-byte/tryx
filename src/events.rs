use std::sync::Arc;

use prost::Message as ProstMessage;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, PyTypeInfo, Python, pyclass, pymethods, types::{PyAnyMethods, PyBytes, PyType, PyTypeMethods}};
use whatsapp_rust::types::events::{LoggedOut as WhatsAppLoggedOut, ConnectFailureReason };
use pyo3::sync::PyOnceLock;
use whatsapp_rust::types::message::{MessageInfo as WhatsappMessageInfo};
use tracing::{debug, info};
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
struct Connected;

#[pyclass]
struct Disconnected;


#[pyclass]
struct LoggedOut{
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
struct PairSuccess {
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
struct PairError {
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
struct QrScannedWithoutMultidevice;


#[pyclass]
struct ClientOutDated;

#[pyclass]
pub struct Message {
    inner: Box<waproto::whatsapp::Message>,
    #[pyo3(get)]
    message_info: MessageInfo,
}
impl Message {
    pub fn new(inner: Box<waproto::whatsapp::Message>, message_info: WhatsappMessageInfo) -> Self {
        let info = MessageInfo {
            inner: Arc::new(message_info.clone()),
            id: message_info.id.clone(),
            r#type: message_info.r#type.clone(),
            push_name: message_info.push_name.clone(),
        };
        Self { inner, message_info: info }
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
    fn raw_proto(&self, py: Python) -> PyResult<Py<PyAny>> {
        let mut buffer = Vec::new();
        self.inner
            .as_ref()
            .encode(&mut buffer)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let proto = parse_message_proto(py, &buffer)?;
        Ok(proto)
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
pub struct Dispatcher {
    connected: Vec<Py<PyAny>>,
    disconnected: Vec<Py<PyAny>>,
    logged_out: Vec<Py<PyAny>>,
    pair_success: Vec<Py<PyAny>>,
    pair_error: Vec<Py<PyAny>>,
    pairing_qr_code: Vec<Py<PyAny>>,
    pairing_code: Vec<Py<PyAny>>,
    qr_scanned_without_multidevice: Vec<Py<PyAny>>,
    client_outdated: Vec<Py<PyAny>>,
    message: Vec<Py<PyAny>>,
    receipt: Vec<Py<PyAny>>,
    undecryptable_message: Vec<Py<PyAny>>,
    notification: Vec<Py<PyAny>>,
    chat_presence: Vec<Py<PyAny>>,
    presence: Vec<Py<PyAny>>,
    picture_update: Vec<Py<PyAny>>,
    user_about_update: Vec<Py<PyAny>>,
    joined_group: Vec<Py<PyAny>>,
    group_info_update: Vec<Py<PyAny>>,
    contact_update: Vec<Py<PyAny>>,
    push_name_update: Vec<Py<PyAny>>,
    self_push_name_update: Vec<Py<PyAny>>,
    pin_update: Vec<Py<PyAny>>,
    mute_update: Vec<Py<PyAny>>,
    archive_update: Vec<Py<PyAny>>,
    mark_chat_as_read_update: Vec<Py<PyAny>>,
    history_sync: Vec<Py<PyAny>>,
    offline_sync_preview: Vec<Py<PyAny>>,
    offline_sync_completed: Vec<Py<PyAny>>,
    device_list_update: Vec<Py<PyAny>>,
    business_status_update: Vec<Py<PyAny>>,
    stream_replaced: Vec<Py<PyAny>>,
    temporary_ban: Vec<Py<PyAny>>,
    connect_failure: Vec<Py<PyAny>>,
    stream_error: Vec<Py<PyAny>>,
    pending_event: Option<DispatchEvent>,
}

#[derive(Clone, Copy)]
enum DispatchEvent {
    Connected,
    Disconnected,
    LoggedOut,
    PairSuccess,
    PairError,
    PairingQrCode,
    PairingCode,
    QrScannedWithoutMultidevice,
    ClientOutDated,
    Message,
}

impl Dispatcher {
    pub fn empty() -> Self {
        Self {
            connected: Vec::new(),
            disconnected: Vec::new(),
            logged_out: Vec::new(),
            pair_success: Vec::new(),
            pair_error: Vec::new(),
            pairing_qr_code: Vec::new(),
            pairing_code: Vec::new(),
            qr_scanned_without_multidevice: Vec::new(),
            client_outdated: Vec::new(),
            message: Vec::new(),
            receipt: Vec::new(),
            undecryptable_message: Vec::new(),
            notification: Vec::new(),
            chat_presence: Vec::new(),
            presence: Vec::new(),
            picture_update: Vec::new(),
            user_about_update: Vec::new(),
            joined_group: Vec::new(),
            group_info_update: Vec::new(),
            contact_update: Vec::new(),
            push_name_update: Vec::new(),
            self_push_name_update: Vec::new(),
            pin_update: Vec::new(),
            mute_update: Vec::new(),
            archive_update: Vec::new(),
            mark_chat_as_read_update: Vec::new(),
            history_sync: Vec::new(),
            offline_sync_preview: Vec::new(),
            offline_sync_completed: Vec::new(),
            device_list_update: Vec::new(),
            business_status_update: Vec::new(),
            stream_replaced: Vec::new(),
            temporary_ban: Vec::new(),
            connect_failure: Vec::new(),
            stream_error: Vec::new(),
            pending_event: None,
        }
    }

    pub fn pairing_qr_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = self.pairing_qr_code
            .iter()
            .map(|handler| handler.clone_ref(py))
            .collect::<Vec<_>>();
        debug!(handlers = handlers.len(), "collected pairing QR handlers");
        handlers
    }
    pub fn message_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = self.message
            .iter()
            .map(|handler| handler.clone_ref(py))
            .collect::<Vec<_>>();
        debug!(handlers = handlers.len(), "collected message handlers");
        handlers
    }
    fn handlers_for_event(&self, event: DispatchEvent) -> &Vec<Py<PyAny>> {
        match event {
            DispatchEvent::Connected => &self.connected,
            DispatchEvent::Disconnected => &self.disconnected,
            DispatchEvent::LoggedOut => &self.logged_out,
            DispatchEvent::PairSuccess => &self.pair_success,
            DispatchEvent::PairError => &self.pair_error,
            DispatchEvent::PairingQrCode => &self.pairing_qr_code,
            DispatchEvent::PairingCode => &self.pairing_code,
            DispatchEvent::QrScannedWithoutMultidevice => &self.qr_scanned_without_multidevice,
            DispatchEvent::ClientOutDated => &self.client_outdated,
            DispatchEvent::Message => &self.message,
        }
    }
}

fn dispatch_event_name(event: DispatchEvent) -> &'static str {
    match event {
        DispatchEvent::Connected => "connected",
        DispatchEvent::Disconnected => "disconnected",
        DispatchEvent::LoggedOut => "logged_out",
        DispatchEvent::PairSuccess => "pair_success",
        DispatchEvent::PairError => "pair_error",
        DispatchEvent::PairingQrCode => "pairing_qr_code",
        DispatchEvent::PairingCode => "pairing_code",
        DispatchEvent::QrScannedWithoutMultidevice => "qr_scanned_without_multidevice",
        DispatchEvent::ClientOutDated => "client_outdated",
        DispatchEvent::Message => "message",
    }
}

/// Maps a Python event class into the internal dispatcher event enum.
fn dispatch_event_from_type(py: Python, event_type: &Bound<PyAny>) -> PyResult<DispatchEvent> {
    let event_type = event_type.cast::<PyType>()?;

    if event_type.is_subclass(&Connected::type_object(py))? {
        Ok(DispatchEvent::Connected)
    } else if event_type.is_subclass(&Disconnected::type_object(py))? {
        Ok(DispatchEvent::Disconnected)
    } else if event_type.is_subclass(&LoggedOut::type_object(py))? {
        Ok(DispatchEvent::LoggedOut)
    } else if event_type.is_subclass(&PairSuccess::type_object(py))? {
        Ok(DispatchEvent::PairSuccess)
    } else if event_type.is_subclass(&PairError::type_object(py))? {
        Ok(DispatchEvent::PairError)
    } else if event_type.is_subclass(&PairingQrCode::type_object(py))? {
        Ok(DispatchEvent::PairingQrCode)
    } else if event_type.is_subclass(&PairingCode::type_object(py))? {
        Ok(DispatchEvent::PairingCode)
    } else if event_type.is_subclass(&QrScannedWithoutMultidevice::type_object(py))? {
        Ok(DispatchEvent::QrScannedWithoutMultidevice)
    } else if event_type.is_subclass(&ClientOutDated::type_object(py))? {
        Ok(DispatchEvent::ClientOutDated)
    } else if event_type.is_subclass(&Message::type_object(py))? {
        Ok(DispatchEvent::Message)
    } else {
        Err(pyo3::exceptions::PyValueError::new_err("Unsupported event type"))
    }
}

#[pymethods]
impl Dispatcher {
    #[new]
    fn new() -> Self {
        Self::empty()
    }

    /// Selects an event class and returns a callable decorator.
    ///
    /// Python usage:
    /// @dispatcher.on(Connected)
    /// def handler(client, event):
    ///     ...
    fn on(slf: Py<Self>, py: Python, event_type: &Bound<PyAny>) -> PyResult<Py<PyAny>> {
        let event = dispatch_event_from_type(py, event_type)?;
        info!(event = dispatch_event_name(event), "selected event for next callback registration");
        {
            let mut this = slf.borrow_mut(py);
            this.pending_event = Some(event);
        }
        Ok(slf.into_any())
    }

    /// Registers the function produced by the decorator call and returns it.
    fn __call__(&mut self, py: Python, func: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let event = self
            .pending_event
            .take()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("on(event_type) must be called before registering callback"))?;

        match event {
            DispatchEvent::Connected => self.connected.push(func.clone_ref(py)),
            DispatchEvent::Disconnected => self.disconnected.push(func.clone_ref(py)),
            DispatchEvent::LoggedOut => self.logged_out.push(func.clone_ref(py)),
            DispatchEvent::PairSuccess => self.pair_success.push(func.clone_ref(py)),
            DispatchEvent::PairError => self.pair_error.push(func.clone_ref(py)),
            DispatchEvent::PairingQrCode => self.pairing_qr_code.push(func.clone_ref(py)),
            DispatchEvent::PairingCode => self.pairing_code.push(func.clone_ref(py)),
            DispatchEvent::QrScannedWithoutMultidevice => self.qr_scanned_without_multidevice.push(func.clone_ref(py)),
            DispatchEvent::ClientOutDated => self.client_outdated.push(func.clone_ref(py)),
            DispatchEvent::Message => self.message.push(func.clone_ref(py)),
        }

        let total_handlers = self.handlers_for_event(event).len();
        info!(event = dispatch_event_name(event), handlers = total_handlers, "registered Python callback");

        Ok(func)
    }
}

