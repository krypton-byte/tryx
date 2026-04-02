use pyo3::prelude::*;
use tracing::{debug, info};
use pyo3::types::PyType;
use pyo3::PyTypeInfo;
use super::types::{
    EvArchiveUpdate,
    EvBusinessStatusUpdate,
    EvChatPresence,
    EvClientOutDated,
    EvConnectFailure,
    EvConnected,
    EvContactNumberChanged,
    EvContactSyncRequested,
    EvContactUpdate,
    EvContactUpdated,
    EvDeleteChatUpdate,
    EvDeleteMessageForMeUpdate,
    EvDeviceListUpdate,
    EvDisappearingModeChanged,
    EvDisconnected,
    EvGroupInfoUpdate,
    EvHistorySync,
    EvJoinedGroup,
    EvLoggedOut,
    EvMarkChatAsReadUpdate,
    EvMessage,
    EvMuteUpdate,
    EvNewsletterLiveUpdate,
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
    EvStarUpdate,
    EvStreamError,
    EvStreamReplaced,
    EvTemporaryBan,
    EvUndecryptableMessage,
    EvUserAboutUpdate,
};
use crate::exceptions::UnsupportedEventType;

/// Generates the `Dispatcher` struct, `DispatchEvent` enum, and all associated
/// boilerplate from a single declarative list.
///
/// Each entry maps:
///   field, EnumVariant, "wire_name", EvPyClass, "log label", handler_fn
macro_rules! define_dispatcher {
    (
        $(
            $field:ident, $variant:ident, $name:expr, $ev_type:ty, $log_label:expr, $handler_fn:ident
        );+ $(;)?
    ) => {
        #[pyclass]
        pub struct Dispatcher {
            $( $field: Vec<Py<PyAny>>, )+
            pending_event: Option<DispatchEvent>,
        }

        #[derive(Clone, Copy)]
        enum DispatchEvent {
            $( $variant, )+
        }

        impl Dispatcher {
            fn cloned_handlers(py: Python<'_>, handlers: &[Py<PyAny>]) -> Vec<Py<PyAny>> {
                handlers.iter().map(|h| h.clone_ref(py)).collect()
            }

            pub fn empty() -> Self {
                Self {
                    $( $field: Vec::new(), )+
                    pending_event: None,
                }
            }

            $(
                pub fn $handler_fn(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
                    let handlers = Self::cloned_handlers(py, &self.$field);
                    debug!(handlers = handlers.len(), concat!("collected ", $log_label, " handlers"));
                    handlers
                }
            )+

            /// Alias kept for backward compatibility.
            pub fn logout_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
                self.logged_out_handlers(py)
            }

            fn handlers_for_event(&self, event: DispatchEvent) -> &Vec<Py<PyAny>> {
                match event {
                    $( DispatchEvent::$variant => &self.$field, )+
                }
            }
        }

        fn dispatch_event_name(event: DispatchEvent) -> &'static str {
            match event {
                $( DispatchEvent::$variant => $name, )+
            }
        }

        fn dispatch_event_from_type(py: Python, event_type: &Bound<PyAny>) -> PyResult<DispatchEvent> {
            let event_type = event_type.cast::<PyType>()?;
            $(
                if event_type.is_subclass(&<$ev_type>::type_object(py))? {
                    return Ok(DispatchEvent::$variant);
                }
            )+
            Err(PyErr::new::<UnsupportedEventType, _>("Unsupported event type"))
        }

        #[pymethods]
        impl Dispatcher {
            #[new]
            fn new() -> Self {
                Self::empty()
            }

            /// Returns a callable decorator for the given event class.
            ///
            /// ```python
            /// @dispatcher.on(EvMessage)
            /// async def handler(client, event): ...
            /// ```
            fn on(slf: Py<Self>, py: Python, event_type: &Bound<PyAny>) -> PyResult<Py<PyAny>> {
                let event = dispatch_event_from_type(py, event_type)?;
                info!(event = dispatch_event_name(event), "selected event for next callback registration");
                {
                    let mut this = slf.borrow_mut(py);
                    this.pending_event = Some(event);
                }
                Ok(slf.into_any())
            }

            /// Registers the decorated function and returns it unchanged.
            fn __call__(&mut self, py: Python, func: Py<PyAny>) -> PyResult<Py<PyAny>> {
                let event = self
                    .pending_event
                    .take()
                    .ok_or_else(|| {
                        pyo3::exceptions::PyRuntimeError::new_err(
                            "on(event_type) must be called before registering callback",
                        )
                    })?;

                match event {
                    $( DispatchEvent::$variant => self.$field.push(func.clone_ref(py)), )+
                }

                let total_handlers = self.handlers_for_event(event).len();
                info!(event = dispatch_event_name(event), handlers = total_handlers, "registered Python callback");

                Ok(func)
            }
        }
    };
}

// ── Single source of truth for all dispatcher events ─────────────────────
// field, EnumVariant, "wire_name", EvPyClass, "log label", handler_method
define_dispatcher! {
    connected,                       Connected,                     "connected",                       EvConnected,                     "connected",                      connected_handlers;
    disconnected,                    Disconnected,                  "disconnected",                    EvDisconnected,                  "disconnected",                   disconnected_handlers;
    logged_out,                      LoggedOut,                     "logged_out",                      EvLoggedOut,                     "logged out",                     logged_out_handlers;
    pair_success,                    PairSuccess,                   "pair_success",                    EvPairSuccess,                   "pair success",                   pair_success_handlers;
    pair_error,                      PairError,                     "pair_error",                      EvPairError,                     "pair error",                     pair_error_handlers;
    pairing_qr_code,                 PairingQrCode,                 "pairing_qr_code",                 EvPairingQrCode,                 "pairing QR",                     pairing_qr_handlers;
    pairing_code,                    PairingCode,                   "pairing_code",                    EvPairingCode,                   "pairing code",                   pairing_code_handlers;
    qr_scanned_without_multidevice,  QrScannedWithoutMultidevice,   "qr_scanned_without_multidevice",  EvQrScannedWithoutMultidevice,   "QR scanned without multidevice", qr_scanned_without_multidevice_handlers;
    client_outdated,                 ClientOutDated,                "client_outdated",                 EvClientOutDated,                "client outdated",                client_outdated_handlers;
    message,                         Message,                       "message",                         EvMessage,                       "message",                        message_handlers;
    receipt,                         Receipt,                       "receipt",                         EvReceipt,                       "receipt",                        receipt_handlers;
    undecryptable_message,           UndecryptableMessage,          "undecryptable_message",           EvUndecryptableMessage,          "undecryptable message",          undecryptable_message_handlers;
    notification,                    Notification,                  "notification",                    EvNotification,                  "notification",                   notification_handlers;
    chat_presence,                   ChatPresence,                  "chat_presence",                   EvChatPresence,                  "chat presence",                  chat_presence_handlers;
    presence,                        Presence,                      "presence",                        EvPresence,                      "presence",                       presence_handlers;
    picture_update,                  PictureUpdate,                 "picture_update",                  EvPictureUpdate,                 "picture update",                 picture_update_handlers;
    user_about_update,               UserAboutUpdate,               "user_about_update",               EvUserAboutUpdate,               "user about update",              user_about_update_handlers;
    joined_group,                    JoinedGroup,                   "joined_group",                    EvJoinedGroup,                   "joined group",                   joined_group_handlers;
    group_info_update,               GroupInfoUpdate,               "group_info_update",               EvGroupInfoUpdate,               "group info update",              group_info_update_handlers;
    contact_update,                  ContactUpdate,                 "contact_update",                  EvContactUpdate,                 "contact update",                 contact_update_handlers;
    push_name_update,                PushNameUpdate,                "push_name_update",                EvPushNameUpdate,                "push name update",               push_name_update_handlers;
    self_push_name_update,           SelfPushNameUpdated,           "self_push_name_updated",          EvSelfPushNameUpdated,           "self push name updated",         self_push_name_updated_handlers;
    pin_update,                      PinUpdate,                     "pin_update",                      EvPinUpdate,                     "pin update",                     pin_update_handlers;
    mute_update,                     MuteUpdate,                    "mute_update",                     EvMuteUpdate,                    "mute update",                    mute_update_handlers;
    archive_update,                  ArchiveUpdate,                 "archive_update",                  EvArchiveUpdate,                 "archive update",                 archive_update_handlers;
    mark_chat_as_read_update,        MarkChatAsReadUpdate,          "mark_chat_as_read_update",        EvMarkChatAsReadUpdate,          "mark chat as read",              mark_chat_as_read_update_handlers;
    history_sync,                    HistorySync,                   "history_sync",                    EvHistorySync,                   "history sync",                   history_sync_handlers;
    offline_sync_preview,            OfflineSyncPreview,            "offline_sync_preview",            EvOfflineSyncPreview,            "offline sync preview",           offline_sync_preview_handlers;
    offline_sync_completed,          OfflineSyncCompleted,          "offline_sync_completed",          EvOfflineSyncCompleted,          "offline sync completed",         offline_sync_completed_handlers;
    device_list_update,              DeviceListUpdate,              "device_list_update",              EvDeviceListUpdate,              "device list update",             device_list_update_handlers;
    business_status_update,          BusinessStatusUpdate,          "business_status_update",          EvBusinessStatusUpdate,          "business status update",         business_status_update_handlers;
    stream_replaced,                 StreamReplaced,                "stream_replaced",                 EvStreamReplaced,                "stream replaced",                stream_replaced_handlers;
    temporary_ban,                   TemporaryBan,                  "temporary_ban",                   EvTemporaryBan,                  "temporary ban",                  temporary_ban_handlers;
    connect_failure,                 ConnectFailure,                "connect_failure",                 EvConnectFailure,                "connect failure",                connect_failure_handlers;
    stream_error,                    StreamError,                   "stream_error",                    EvStreamError,                   "stream error",                   stream_error_handlers;
    contact_number_changed,          ContactNumberChanged,          "contact_number_changed",          EvContactNumberChanged,          "contact number changed",         contact_number_changed_handlers;
    disappearing_mode_changed,       DisappearingModeChanged,       "disappearing_mode_changed",       EvDisappearingModeChanged,       "disappearing mode changed",      disappearing_mode_changed_handlers;
    contact_sync_requested,          ContactSyncRequested,          "contact_sync_requested",          EvContactSyncRequested,          "contact sync requested",         contact_sync_requested_handlers;
    contact_updated,                 ContactUpdated,                "contact_updated",                 EvContactUpdated,                "contact updated",                contact_updated_handlers;
    star_update,                     StarUpdate,                    "star_update",                     EvStarUpdate,                    "star update",                    star_update_handlers;
    newsletter_live_update,          NewsletterLiveUpdate,          "newsletter_live_update",          EvNewsletterLiveUpdate,          "newsletter live update",         newsletter_live_update_handlers;
    delete_chat_update,              DeleteChatUpdate,              "delete_chat_update",              EvDeleteChatUpdate,              "delete chat update",             delete_chat_update_handlers;
    delete_message_for_me_update,    DeleteMessageForMeUpdate,      "delete_message_for_me_update",    EvDeleteMessageForMeUpdate,      "delete message for me update",   delete_message_for_me_update_handlers;
}
