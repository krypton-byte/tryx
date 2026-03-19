use pyo3::prelude::*;

use self::clients::tryx_client::TryxClient;
use self::clients::tryx::Tryx;
use self::events::types::{
    EvArchiveUpdate,
    EvBusinessStatusUpdate,
    EvChatPresence,
    EvClientOutDated,
    EvConnectFailure,
    EvConnected,
    EvContactUpdate,
    EvDeviceListUpdate,
    EvDisconnected,
    EvGroupInfoUpdate,
    EvHistorySync,
    EvJoinedGroup,
    EvLoggedOut,
    EvMarkChatAsReadUpdate,
    EvMessage,
    EvMuteUpdate,
    EvNotification,
    EvOfflineSyncCompleted,
    EvOfflineSyncPreview,
    EvPairError,
    EvPairSuccess,
    EvPairingCode,
    EvPairingQrCode,
    EvPictureUpdate,
    EvPinUpdate,
    EvPresence,
    EvPushNameUpdate,
    EvQrScannedWithoutMultidevice,
    EvReceipt,
    EvSelfPushNameUpdated,
    EvStreamError,
    EvStreamReplaced,
    EvTemporaryBan,
    EvUndecryptableMessage,
    EvUserAboutUpdate,
};
use self::backend::SqliteBackend;
use self::exceptions::{EventDispatchError, FailedBuildBot, PyPayloadBuildError, UnsupportedBackend, UnsupportedEventType};
use self::types::{JID, MessageInfo, UploadResponse};
use self::wacore::download::MediaType;

/// A Python module implemented in Rust.
/// 
#[pymodule]
fn _tryx(_py: &Bound<PyModule>) -> PyResult<()> {
    // m.
    let client_module = PyModule::new(_py.py(), "client")?;
    client_module.add_class::<TryxClient>()?;
    client_module.add_class::<Tryx>()?;
    _py.add_submodule(&client_module)?;

    let events_module = PyModule::new(_py.py(), "events")?;
    events_module.add_class::<EvMessage>()?;
    events_module.add_class::<EvPairingQrCode>()?;
    events_module.add_class::<EvClientOutDated>()?;
    events_module.add_class::<EvQrScannedWithoutMultidevice>()?;
    events_module.add_class::<EvPairError>()?;
    events_module.add_class::<EvPairSuccess>()?;
    events_module.add_class::<EvLoggedOut>()?;
    events_module.add_class::<EvDisconnected>()?;
    events_module.add_class::<EvConnected>()?;
    events_module.add_class::<EvPairingCode>()?;
    events_module.add_class::<EvReceipt>()?;
    events_module.add_class::<EvUndecryptableMessage>()?;
    events_module.add_class::<EvNotification>()?;
    events_module.add_class::<EvChatPresence>()?;
    events_module.add_class::<EvPresence>()?;
    events_module.add_class::<EvPictureUpdate>()?;
    events_module.add_class::<EvUserAboutUpdate>()?;
    events_module.add_class::<EvJoinedGroup>()?;
    events_module.add_class::<EvGroupInfoUpdate>()?;
    events_module.add_class::<EvContactUpdate>()?;
    events_module.add_class::<EvPushNameUpdate>()?;
    events_module.add_class::<EvSelfPushNameUpdated>()?;
    events_module.add_class::<EvPinUpdate>()?;
    events_module.add_class::<EvMuteUpdate>()?;
    events_module.add_class::<EvArchiveUpdate>()?;
    events_module.add_class::<EvMarkChatAsReadUpdate>()?;
    events_module.add_class::<EvHistorySync>()?;
    events_module.add_class::<EvOfflineSyncPreview>()?;
    events_module.add_class::<EvOfflineSyncCompleted>()?;
    events_module.add_class::<EvDeviceListUpdate>()?;
    events_module.add_class::<EvBusinessStatusUpdate>()?;
    events_module.add_class::<EvStreamReplaced>()?;
    events_module.add_class::<EvTemporaryBan>()?;
    events_module.add_class::<EvConnectFailure>()?;
    events_module.add_class::<EvStreamError>()?;
    _py.add_submodule(&events_module)?;

    let backend_module = PyModule::new(_py.py(), "backend")?;
    backend_module.add_class::<SqliteBackend>()?;
    _py.add_submodule(&backend_module)?;

    let exceptions_module = PyModule::new(_py.py(), "exceptions")?;
    exceptions_module.add_class::<FailedBuildBot>()?;
    exceptions_module.add_class::<EventDispatchError>()?;
    exceptions_module.add_class::<PyPayloadBuildError>()?;
    exceptions_module.add_class::<UnsupportedBackend>()?;
    exceptions_module.add_class::<UnsupportedEventType>()?;
    _py.add_submodule(&exceptions_module)?;

    let types_module = PyModule::new(_py.py(), "types")?;
    types_module.add_class::<JID>()?;
    types_module.add_class::<MessageInfo>()?;
    types_module.add_class::<UploadResponse>()?;
    _py.add_submodule(&types_module)?;
    let wacore_module = PyModule::new(_py.py(), "wacore")?;
    wacore_module.add_class::<MediaType>()?;
    _py.add_submodule(&wacore_module)?;
    Ok(())
}

mod backend;
mod clients;
mod events;
mod types;
mod exceptions;
mod wacore;
mod log;