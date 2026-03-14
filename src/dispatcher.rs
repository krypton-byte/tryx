use pyo3::prelude::*;
use tracing::{debug, info};
use pyo3::types::{PyType};
use pyo3::{PyTypeInfo};
use crate::events::{Connected, Disconnected, LoggedOut, PairSuccess, PairError, PairingCode, PairingQrCode, QrScannedWithoutMultidevice, ClientOutDated, Message};
use crate::exceptions::UnsupportedEventType;

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
    pub fn logout_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = self.logged_out
            .iter()
            .map(|handler| handler.clone_ref(py))
            .collect::<Vec<_>>();
        debug!(handlers = handlers.len(), "collected logged out handlers");
        handlers
    }
    pub fn conneccted_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = self.connected
            .iter()
            .map(|handler| handler.clone_ref(py))
            .collect::<Vec<_>>();
        debug!(handlers = handlers.len(), "collected connected handlers");
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
        Err(PyErr::new::<UnsupportedEventType, _>("Unsupported event type"))
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

