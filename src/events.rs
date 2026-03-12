use pyo3::pyclass;
use whatsapp_rust::types::events::{LoggedOut as WhatsAppLoggedOut, ConnectFailureReason};
use crate::types::{JID};
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
    business_name: String,
    platform: String,
}

#[pyclass]
struct PairError {
    id: JID,
    lid: JID,
    business_name: String,
    platform: String,
    error: String,
}

#[pyclass]
struct PairingQrCode {
    code: String,
    timeout: u64,
}

#[pyclass]
struct PairingCode {
    code: String,
    timeout: u64,
}

#[pyclass]
struct QrScannedWithoutMultidevice;


#[pyclass]
struct ClientOutDated;

#[pyclass]
struct Message {
    
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
