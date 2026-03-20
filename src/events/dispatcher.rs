use pyo3::prelude::*;
use tracing::{debug, info};
use pyo3::types::{PyType};
use pyo3::{PyTypeInfo};
use super::types::{
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
use crate::events::types::{EvContactSyncRequested, EvContactUpdated, EvDisappearingModeChanged, EvNewsletterLiveUpdate, EvStarUpdate};
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
    disappearing_mode_changed: Vec<Py<PyAny>>,
    contact_sync_requested: Vec<Py<PyAny>>,
    contact_updated: Vec<Py<PyAny>>,
    star_update: Vec<Py<PyAny>>,
    newsletter_live_update: Vec<Py<PyAny>>,
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
    Receipt,
    UndecryptableMessage,
    Notification,
    ChatPresence,
    Presence,
    PictureUpdate,
    UserAboutUpdate,
    JoinedGroup,
    GroupInfoUpdate,
    ContactUpdate,
    PushNameUpdate,
    SelfPushNameUpdated,
    PinUpdate,
    MuteUpdate,
    ArchiveUpdate,
    MarkChatAsReadUpdate,
    HistorySync,
    OfflineSyncPreview,
    OfflineSyncCompleted,
    DeviceListUpdate,
    BusinessStatusUpdate,
    StreamReplaced,
    TemporaryBan,
    ConnectFailure,
    StreamError,
    DisappearingModeChanged,
    ContactSyncRequested,
    ContactUpdated,
    StarUpdate,
    NewsletterLiveUpdate,
}

impl Dispatcher {
    fn cloned_handlers(py: Python<'_>, handlers: &Vec<Py<PyAny>>) -> Vec<Py<PyAny>> {
        handlers
            .iter()
            .map(|handler| handler.clone_ref(py))
            .collect::<Vec<_>>()
    }

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
            disappearing_mode_changed: Vec::new(),
            pending_event: None,
            contact_sync_requested: Vec::new(),
            contact_updated: Vec::new(),
            star_update: Vec::new(),
            newsletter_live_update: Vec::new(),
        }
    }

    pub fn pairing_qr_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.pairing_qr_code);
        debug!(handlers = handlers.len(), "collected pairing QR handlers");
        handlers
    }

    pub fn message_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.message);
        debug!(handlers = handlers.len(), "collected message handlers");
        handlers
    }

    pub fn logout_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.logged_out);
        debug!(handlers = handlers.len(), "collected logged out handlers");
        handlers
    }

    pub fn logged_out_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        self.logout_handlers(py)
    }

    pub fn connected_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.connected);
        debug!(handlers = handlers.len(), "collected connected handlers");
        handlers
    }

    pub fn disconnected_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.disconnected);
        debug!(handlers = handlers.len(), "collected disconnected handlers");
        handlers
    }

    pub fn pair_success_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.pair_success);
        debug!(handlers = handlers.len(), "collected pair success handlers");
        handlers
    }

    pub fn pair_error_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.pair_error);
        debug!(handlers = handlers.len(), "collected pair error handlers");
        handlers
    }

    pub fn pairing_code_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.pairing_code);
        debug!(handlers = handlers.len(), "collected pairing code handlers");
        handlers
    }

    pub fn qr_scanned_without_multidevice_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.qr_scanned_without_multidevice);
        debug!(handlers = handlers.len(), "collected QR scanned without multidevice handlers");
        handlers
    }

    pub fn client_outdated_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.client_outdated);
        debug!(handlers = handlers.len(), "collected client outdated handlers");
        handlers
    }

    pub fn newsletter_live_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.newsletter_live_update);
        debug!(handlers = handlers.len(), "collected newsletter live update handlers");
        handlers
    }

    pub fn receipt_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.receipt);
        debug!(handlers = handlers.len(), "collected receipt handlers");
        handlers
    }

    pub fn undecryptable_message_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.undecryptable_message);
        debug!(handlers = handlers.len(), "collected undecryptable message handlers");
        handlers
    }

    pub fn notification_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.notification);
        debug!(handlers = handlers.len(), "collected notification handlers");
        handlers
    }

    pub fn chat_presence_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.chat_presence);
        debug!(handlers = handlers.len(), "collected chat presence handlers");
        handlers
    }

    pub fn presence_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.presence);
        debug!(handlers = handlers.len(), "collected presence handlers");
        handlers
    }

    pub fn picture_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.picture_update);
        debug!(handlers = handlers.len(), "collected picture update handlers");
        handlers
    }

    pub fn user_about_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.user_about_update);
        debug!(handlers = handlers.len(), "collected user about update handlers");
        handlers
    }

    pub fn joined_group_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.joined_group);
        debug!(handlers = handlers.len(), "collected joined group handlers");
        handlers
    }

    pub fn group_info_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.group_info_update);
        debug!(handlers = handlers.len(), "collected group info update handlers");
        handlers
    }

    pub fn contact_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.contact_update);
        debug!(handlers = handlers.len(), "collected contact update handlers");
        handlers
    }

    pub fn push_name_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.push_name_update);
        debug!(handlers = handlers.len(), "collected push name update handlers");
        handlers
    }

    pub fn self_push_name_updated_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.self_push_name_update);
        debug!(handlers = handlers.len(), "collected self push name updated handlers");
        handlers
    }

    pub fn pin_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.pin_update);
        debug!(handlers = handlers.len(), "collected pin update handlers");
        handlers
    }

    pub fn mute_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.mute_update);
        debug!(handlers = handlers.len(), "collected mute update handlers");
        handlers
    }

    pub fn archive_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.archive_update);
        debug!(handlers = handlers.len(), "collected archive update handlers");
        handlers
    }

    pub fn mark_chat_as_read_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.mark_chat_as_read_update);
        debug!(handlers = handlers.len(), "collected mark chat as read handlers");
        handlers
    }

    pub fn history_sync_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.history_sync);
        debug!(handlers = handlers.len(), "collected history sync handlers");
        handlers
    }

    pub fn offline_sync_preview_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.offline_sync_preview);
        debug!(handlers = handlers.len(), "collected offline sync preview handlers");
        handlers
    }

    pub fn offline_sync_completed_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.offline_sync_completed);
        debug!(handlers = handlers.len(), "collected offline sync completed handlers");
        handlers
    }

    pub fn device_list_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.device_list_update);
        debug!(handlers = handlers.len(), "collected device list update handlers");
        handlers
    }

    pub fn business_status_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.business_status_update);
        debug!(handlers = handlers.len(), "collected business status update handlers");
        handlers
    }

    pub fn stream_replaced_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.stream_replaced);
        debug!(handlers = handlers.len(), "collected stream replaced handlers");
        handlers
    }

    pub fn temporary_ban_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.temporary_ban);
        debug!(handlers = handlers.len(), "collected temporary ban handlers");
        handlers
    }

    pub fn connect_failure_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.connect_failure);
        debug!(handlers = handlers.len(), "collected connect failure handlers");
        handlers
    }

    pub fn stream_error_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.stream_error);
        debug!(handlers = handlers.len(), "collected stream error handlers");
        handlers
    }
    pub fn disappearing_mode_changed_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.disappearing_mode_changed);
        debug!(handlers = handlers.len(), "collected disappearing mode changed handlers");
        handlers
    }
    pub fn contact_number_changed_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.contact_update);
        debug!(handlers = handlers.len(), "collected contact number changed handlers");
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
            DispatchEvent::Receipt => &self.receipt,
            DispatchEvent::UndecryptableMessage => &self.undecryptable_message,
            DispatchEvent::Notification => &self.notification,
            DispatchEvent::ChatPresence => &self.chat_presence,
            DispatchEvent::Presence => &self.presence,
            DispatchEvent::PictureUpdate => &self.picture_update,
            DispatchEvent::UserAboutUpdate => &self.user_about_update,
            DispatchEvent::JoinedGroup => &self.joined_group,
            DispatchEvent::GroupInfoUpdate => &self.group_info_update,
            DispatchEvent::ContactUpdate => &self.contact_update,
            DispatchEvent::PushNameUpdate => &self.push_name_update,
            DispatchEvent::SelfPushNameUpdated => &self.self_push_name_update,
            DispatchEvent::PinUpdate => &self.pin_update,
            DispatchEvent::MuteUpdate => &self.mute_update,
            DispatchEvent::ArchiveUpdate => &self.archive_update,
            DispatchEvent::MarkChatAsReadUpdate => &self.mark_chat_as_read_update,
            DispatchEvent::HistorySync => &self.history_sync,
            DispatchEvent::OfflineSyncPreview => &self.offline_sync_preview,
            DispatchEvent::OfflineSyncCompleted => &self.offline_sync_completed,
            DispatchEvent::DeviceListUpdate => &self.device_list_update,
            DispatchEvent::BusinessStatusUpdate => &self.business_status_update,
            DispatchEvent::StreamReplaced => &self.stream_replaced,
            DispatchEvent::TemporaryBan => &self.temporary_ban,
            DispatchEvent::ConnectFailure => &self.connect_failure,
            DispatchEvent::StreamError => &self.stream_error,
            DispatchEvent::DisappearingModeChanged => &self.disappearing_mode_changed,
            DispatchEvent::ContactSyncRequested => &self.contact_sync_requested,
            DispatchEvent::ContactUpdated => &self.contact_updated,
            DispatchEvent::StarUpdate => &self.star_update,
            DispatchEvent::NewsletterLiveUpdate => &self.newsletter_live_update,
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
        DispatchEvent::Receipt => "receipt",
        DispatchEvent::UndecryptableMessage => "undecryptable_message",
        DispatchEvent::Notification => "notification",
        DispatchEvent::ChatPresence => "chat_presence",
        DispatchEvent::Presence => "presence",
        DispatchEvent::PictureUpdate => "picture_update",
        DispatchEvent::UserAboutUpdate => "user_about_update",
        DispatchEvent::JoinedGroup => "joined_group",
        DispatchEvent::GroupInfoUpdate => "group_info_update",
        DispatchEvent::ContactUpdate => "contact_update",
        DispatchEvent::PushNameUpdate => "push_name_update",
        DispatchEvent::SelfPushNameUpdated => "self_push_name_updated",
        DispatchEvent::PinUpdate => "pin_update",
        DispatchEvent::MuteUpdate => "mute_update",
        DispatchEvent::ArchiveUpdate => "archive_update",
        DispatchEvent::MarkChatAsReadUpdate => "mark_chat_as_read_update",
        DispatchEvent::HistorySync => "history_sync",
        DispatchEvent::OfflineSyncPreview => "offline_sync_preview",
        DispatchEvent::OfflineSyncCompleted => "offline_sync_completed",
        DispatchEvent::DeviceListUpdate => "device_list_update",
        DispatchEvent::BusinessStatusUpdate => "business_status_update",
        DispatchEvent::StreamReplaced => "stream_replaced",
        DispatchEvent::TemporaryBan => "temporary_ban",
        DispatchEvent::ConnectFailure => "connect_failure",
        DispatchEvent::StreamError => "stream_error",
        DispatchEvent::DisappearingModeChanged => "disappearing_mode_changed",
        DispatchEvent::ContactSyncRequested => "contact_sync_requested",
        DispatchEvent::ContactUpdated => "contact_updated",
        DispatchEvent::StarUpdate => "star_update",
        DispatchEvent::NewsletterLiveUpdate => "newsletter_live_update",
    }
}

/// Maps a Python event class into the internal dispatcher event enum.
fn dispatch_event_from_type(py: Python, event_type: &Bound<PyAny>) -> PyResult<DispatchEvent> {
    let event_type = event_type.cast::<PyType>()?;

    if event_type.is_subclass(&EvConnected::type_object(py))? {
        Ok(DispatchEvent::Connected)
    } else if event_type.is_subclass(&EvDisconnected::type_object(py))? {
        Ok(DispatchEvent::Disconnected)
    } else if event_type.is_subclass(&EvLoggedOut::type_object(py))? {
        Ok(DispatchEvent::LoggedOut)
    } else if event_type.is_subclass(&EvPairSuccess::type_object(py))? {
        Ok(DispatchEvent::PairSuccess)
    } else if event_type.is_subclass(&EvPairError::type_object(py))? {
        Ok(DispatchEvent::PairError)
    } else if event_type.is_subclass(&EvPairingQrCode::type_object(py))? {
        Ok(DispatchEvent::PairingQrCode)
    } else if event_type.is_subclass(&EvPairingCode::type_object(py))? {
        Ok(DispatchEvent::PairingCode)
    } else if event_type.is_subclass(&EvQrScannedWithoutMultidevice::type_object(py))? {
        Ok(DispatchEvent::QrScannedWithoutMultidevice)
    } else if event_type.is_subclass(&EvClientOutDated::type_object(py))? {
        Ok(DispatchEvent::ClientOutDated)
    } else if event_type.is_subclass(&EvMessage::type_object(py))? {
        Ok(DispatchEvent::Message)
    } else if event_type.is_subclass(&EvReceipt::type_object(py))? {
        Ok(DispatchEvent::Receipt)
    } else if event_type.is_subclass(&EvUndecryptableMessage::type_object(py))? {
        Ok(DispatchEvent::UndecryptableMessage)
    } else if event_type.is_subclass(&EvNotification::type_object(py))? {
        Ok(DispatchEvent::Notification)
    } else if event_type.is_subclass(&EvChatPresence::type_object(py))? {
        Ok(DispatchEvent::ChatPresence)
    } else if event_type.is_subclass(&EvPresence::type_object(py))? {
        Ok(DispatchEvent::Presence)
    } else if event_type.is_subclass(&EvPictureUpdate::type_object(py))? {
        Ok(DispatchEvent::PictureUpdate)
    } else if event_type.is_subclass(&EvUserAboutUpdate::type_object(py))? {
        Ok(DispatchEvent::UserAboutUpdate)
    } else if event_type.is_subclass(&EvJoinedGroup::type_object(py))? {
        Ok(DispatchEvent::JoinedGroup)
    } else if event_type.is_subclass(&EvGroupInfoUpdate::type_object(py))? {
        Ok(DispatchEvent::GroupInfoUpdate)
    } else if event_type.is_subclass(&EvContactUpdate::type_object(py))? {
        Ok(DispatchEvent::ContactUpdate)
    } else if event_type.is_subclass(&EvPushNameUpdate::type_object(py))? {
        Ok(DispatchEvent::PushNameUpdate)
    } else if event_type.is_subclass(&EvSelfPushNameUpdated::type_object(py))? {
        Ok(DispatchEvent::SelfPushNameUpdated)
    } else if event_type.is_subclass(&EvPinUpdate::type_object(py))? {
        Ok(DispatchEvent::PinUpdate)
    } else if event_type.is_subclass(&EvMuteUpdate::type_object(py))? {
        Ok(DispatchEvent::MuteUpdate)
    } else if event_type.is_subclass(&EvArchiveUpdate::type_object(py))? {
        Ok(DispatchEvent::ArchiveUpdate)
    } else if event_type.is_subclass(&EvMarkChatAsReadUpdate::type_object(py))? {
        Ok(DispatchEvent::MarkChatAsReadUpdate)
    } else if event_type.is_subclass(&EvHistorySync::type_object(py))? {
        Ok(DispatchEvent::HistorySync)
    } else if event_type.is_subclass(&EvOfflineSyncPreview::type_object(py))? {
        Ok(DispatchEvent::OfflineSyncPreview)
    } else if event_type.is_subclass(&EvOfflineSyncCompleted::type_object(py))? {
        Ok(DispatchEvent::OfflineSyncCompleted)
    } else if event_type.is_subclass(&EvDeviceListUpdate::type_object(py))? {
        Ok(DispatchEvent::DeviceListUpdate)
    } else if event_type.is_subclass(&EvBusinessStatusUpdate::type_object(py))? {
        Ok(DispatchEvent::BusinessStatusUpdate)
    } else if event_type.is_subclass(&EvStreamReplaced::type_object(py))? {
        Ok(DispatchEvent::StreamReplaced)
    } else if event_type.is_subclass(&EvTemporaryBan::type_object(py))? {
        Ok(DispatchEvent::TemporaryBan)
    } else if event_type.is_subclass(&EvConnectFailure::type_object(py))? {
        Ok(DispatchEvent::ConnectFailure)
    } else if event_type.is_subclass(&EvStreamError::type_object(py))? {
        Ok(DispatchEvent::StreamError)
    }else if event_type.is_subclass(&EvDisappearingModeChanged::type_object(py))? {
        Ok(DispatchEvent::DisappearingModeChanged)
    } else if event_type.is_subclass(&EvContactSyncRequested::type_object(py))? {
        Ok(DispatchEvent::ContactSyncRequested)
    } else if event_type.is_subclass(&EvContactUpdated::type_object(py))? {
        Ok(DispatchEvent::ContactUpdated)
    } else if event_type.is_subclass(&EvStarUpdate::type_object(py))? {
        Ok(DispatchEvent::StarUpdate)
    } else if event_type.is_subclass(&EvNewsletterLiveUpdate::type_object(py))? {
        Ok(DispatchEvent::NewsletterLiveUpdate)
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
            DispatchEvent::Receipt => self.receipt.push(func.clone_ref(py)),
            DispatchEvent::UndecryptableMessage => self.undecryptable_message.push(func.clone_ref(py)),
            DispatchEvent::Notification => self.notification.push(func.clone_ref(py)),
            DispatchEvent::ChatPresence => self.chat_presence.push(func.clone_ref(py)),
            DispatchEvent::Presence => self.presence.push(func.clone_ref(py)),
            DispatchEvent::PictureUpdate => self.picture_update.push(func.clone_ref(py)),
            DispatchEvent::UserAboutUpdate => self.user_about_update.push(func.clone_ref(py)),
            DispatchEvent::JoinedGroup => self.joined_group.push(func.clone_ref(py)),
            DispatchEvent::GroupInfoUpdate => self.group_info_update.push(func.clone_ref(py)),
            DispatchEvent::ContactUpdate => self.contact_update.push(func.clone_ref(py)),
            DispatchEvent::PushNameUpdate => self.push_name_update.push(func.clone_ref(py)),
            DispatchEvent::SelfPushNameUpdated => self.self_push_name_update.push(func.clone_ref(py)),
            DispatchEvent::PinUpdate => self.pin_update.push(func.clone_ref(py)),
            DispatchEvent::MuteUpdate => self.mute_update.push(func.clone_ref(py)),
            DispatchEvent::ArchiveUpdate => self.archive_update.push(func.clone_ref(py)),
            DispatchEvent::MarkChatAsReadUpdate => self.mark_chat_as_read_update.push(func.clone_ref(py)),
            DispatchEvent::HistorySync => self.history_sync.push(func.clone_ref(py)),
            DispatchEvent::OfflineSyncPreview => self.offline_sync_preview.push(func.clone_ref(py)),
            DispatchEvent::OfflineSyncCompleted => self.offline_sync_completed.push(func.clone_ref(py)),
            DispatchEvent::DeviceListUpdate => self.device_list_update.push(func.clone_ref(py)),
            DispatchEvent::BusinessStatusUpdate => self.business_status_update.push(func.clone_ref(py)),
            DispatchEvent::StreamReplaced => self.stream_replaced.push(func.clone_ref(py)),
            DispatchEvent::TemporaryBan => self.temporary_ban.push(func.clone_ref(py)),
            DispatchEvent::ConnectFailure => self.connect_failure.push(func.clone_ref(py)),
            DispatchEvent::StreamError => self.stream_error.push(func.clone_ref(py)),
            DispatchEvent::DisappearingModeChanged => self.disappearing_mode_changed.push(func.clone_ref(py)),
            DispatchEvent::ContactSyncRequested => self.contact_sync_requested.push(func.clone_ref(py)),
            DispatchEvent::ContactUpdated => self.contact_updated.push(func.clone_ref(py)),
            DispatchEvent::StarUpdate => self.star_update.push(func.clone_ref(py)),
            DispatchEvent::NewsletterLiveUpdate => self.newsletter_live_update.push(func.clone_ref(py)),
        }

        let total_handlers = self.handlers_for_event(event).len();
        info!(event = dispatch_event_name(event), handlers = total_handlers, "registered Python callback");

        Ok(func)
    }
}

impl Dispatcher {
    pub fn contact_sync_requested_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.contact_sync_requested);
        debug!(handlers = handlers.len(), "collected contact sync requested handlers");
        handlers
    }

    pub fn contact_updated_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.contact_updated);
        debug!(handlers = handlers.len(), "collected contact updated handlers");
        handlers
    }

    pub fn star_update_handlers(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let handlers = Self::cloned_handlers(py, &self.star_update);
        debug!(handlers = handlers.len(), "collected star update handlers");
        handlers
    }

}

