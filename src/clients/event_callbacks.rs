use pyo3::{Py, PyAny, Python};
use wacore::types::events::EventKind;

use crate::events::dispatcher::Dispatcher;

pub type CallbackList = Vec<Py<PyAny>>;

pub struct EventCallbacks {
    pub connected: CallbackList,
    pub disconnected: CallbackList,
    pub logout: CallbackList,
    pub pair_success: CallbackList,
    pub pair_error: CallbackList,
    pub pairing_qr: CallbackList,
    pub pairing_code: CallbackList,
    pub qr_scanned_without_multidevice: CallbackList,
    pub client_outdated: CallbackList,
    pub message: CallbackList,
    pub receipt: CallbackList,
    pub undecryptable_message: CallbackList,
    pub notification: CallbackList,
    pub chat_presence: CallbackList,
    pub presence: CallbackList,
    pub picture_update: CallbackList,
    pub user_about_update: CallbackList,
    #[allow(dead_code)] // Reserved for future JoinedGroup event variant
    pub joined_group: CallbackList,
    pub group_info_update: CallbackList,
    pub contact_update: CallbackList,
    pub push_name_update: CallbackList,
    pub self_push_name_updated: CallbackList,
    pub pin_update: CallbackList,
    pub mute_update: CallbackList,
    pub archive_update: CallbackList,
    pub mark_chat_as_read_update: CallbackList,
    pub history_sync: CallbackList,
    pub offline_sync_preview: CallbackList,
    pub offline_sync_completed: CallbackList,
    pub device_list_update: CallbackList,
    pub business_status_update: CallbackList,
    pub stream_replaced: CallbackList,
    pub temporary_ban: CallbackList,
    pub connect_failure: CallbackList,
    pub stream_error: CallbackList,
    pub disappearing_mode_changed: CallbackList,
    pub contact_number_changed: CallbackList,
    pub contact_sync_requested: CallbackList,
    pub contact_updated: CallbackList,
    pub star_update: CallbackList,
    pub newsletter_live_update: CallbackList,
    pub delete_chat_update: CallbackList,
    pub delete_message_for_me_update: CallbackList,
    pub incoming_call: CallbackList,
    pub identity_change: CallbackList,
    pub lock_chat_update: CallbackList,
    pub favorites_update: CallbackList,
    pub status_privacy_update: CallbackList,
    pub unarchive_chats_setting_update: CallbackList,
    pub disable_link_previews_update: CallbackList,
    pub contact_removed: CallbackList,
    pub offline_sync_interrupted: CallbackList,
    pub client_expiration_changed: CallbackList,
    pub server_ack: CallbackList,
    pub app_state_sync_failed: CallbackList,
    pub mex_notification: CallbackList,
}

impl EventCallbacks {
    pub fn from_dispatcher(py: Python<'_>, dispatcher: &Dispatcher) -> Self {
        Self {
            connected: dispatcher.connected_handlers(py),
            disconnected: dispatcher.disconnected_handlers(py),
            logout: dispatcher.logout_handlers(py),
            pair_success: dispatcher.pair_success_handlers(py),
            pair_error: dispatcher.pair_error_handlers(py),
            pairing_qr: dispatcher.pairing_qr_handlers(py),
            pairing_code: dispatcher.pairing_code_handlers(py),
            qr_scanned_without_multidevice: dispatcher.qr_scanned_without_multidevice_handlers(py),
            client_outdated: dispatcher.client_outdated_handlers(py),
            message: dispatcher.message_handlers(py),
            receipt: dispatcher.receipt_handlers(py),
            undecryptable_message: dispatcher.undecryptable_message_handlers(py),
            notification: dispatcher.notification_handlers(py),
            chat_presence: dispatcher.chat_presence_handlers(py),
            presence: dispatcher.presence_handlers(py),
            picture_update: dispatcher.picture_update_handlers(py),
            user_about_update: dispatcher.user_about_update_handlers(py),
            joined_group: dispatcher.joined_group_handlers(py),
            group_info_update: dispatcher.group_info_update_handlers(py),
            contact_update: dispatcher.contact_update_handlers(py),
            push_name_update: dispatcher.push_name_update_handlers(py),
            self_push_name_updated: dispatcher.self_push_name_updated_handlers(py),
            pin_update: dispatcher.pin_update_handlers(py),
            mute_update: dispatcher.mute_update_handlers(py),
            archive_update: dispatcher.archive_update_handlers(py),
            mark_chat_as_read_update: dispatcher.mark_chat_as_read_update_handlers(py),
            history_sync: dispatcher.history_sync_handlers(py),
            offline_sync_preview: dispatcher.offline_sync_preview_handlers(py),
            offline_sync_completed: dispatcher.offline_sync_completed_handlers(py),
            device_list_update: dispatcher.device_list_update_handlers(py),
            business_status_update: dispatcher.business_status_update_handlers(py),
            stream_replaced: dispatcher.stream_replaced_handlers(py),
            temporary_ban: dispatcher.temporary_ban_handlers(py),
            connect_failure: dispatcher.connect_failure_handlers(py),
            stream_error: dispatcher.stream_error_handlers(py),
            disappearing_mode_changed: dispatcher.disappearing_mode_changed_handlers(py),
            contact_number_changed: dispatcher.contact_number_changed_handlers(py),
            contact_sync_requested: dispatcher.contact_sync_requested_handlers(py),
            contact_updated: dispatcher.contact_updated_handlers(py),
            star_update: dispatcher.star_update_handlers(py),
            delete_message_for_me_update: dispatcher.delete_message_for_me_update_handlers(py),
            incoming_call: dispatcher.incoming_call_handlers(py),
            newsletter_live_update: dispatcher.newsletter_live_update_handlers(py),
            delete_chat_update: dispatcher.delete_chat_update_handlers(py),
            identity_change: dispatcher.identity_change_handlers(py),
            lock_chat_update: dispatcher.lock_chat_update_handlers(py),
            favorites_update: dispatcher.favorites_update_handlers(py),
            status_privacy_update: dispatcher.status_privacy_update_handlers(py),
            unarchive_chats_setting_update: dispatcher.unarchive_chats_setting_update_handlers(py),
            disable_link_previews_update: dispatcher.disable_link_previews_update_handlers(py),
            contact_removed: dispatcher.contact_removed_handlers(py),
            offline_sync_interrupted: dispatcher.offline_sync_interrupted_handlers(py),
            client_expiration_changed: dispatcher.client_expiration_changed_handlers(py),
            server_ack: dispatcher.server_ack_handlers(py),
            app_state_sync_failed: dispatcher.app_state_sync_failed_handlers(py),
            mex_notification: dispatcher.mex_notification_handlers(py),
        }
    }

    /// Returns the exact set of `EventKind`s that have at least one registered Python handler.
    /// Passing this to `Bot::builder().on_event_for(...)` avoids dispatching or cloning
    /// events that Python is not listening to.
    pub fn event_interests(&self) -> Vec<EventKind> {
        let mut kinds = Vec::new();
        macro_rules! push_if {
            ($field:ident, $kind:expr) => {
                if !self.$field.is_empty() {
                    kinds.push($kind);
                }
            };
        }
        push_if!(connected, EventKind::Connected);
        push_if!(disconnected, EventKind::Disconnected);
        push_if!(logout, EventKind::LoggedOut);
        push_if!(pair_success, EventKind::PairSuccess);
        push_if!(pair_error, EventKind::PairError);
        push_if!(pairing_qr, EventKind::PairingQrCode);
        push_if!(pairing_code, EventKind::PairingCode);
        push_if!(qr_scanned_without_multidevice, EventKind::QrScannedWithoutMultidevice);
        push_if!(client_outdated, EventKind::ClientOutdated);
        push_if!(message, EventKind::Messages);
        push_if!(receipt, EventKind::Receipt);
        push_if!(undecryptable_message, EventKind::UndecryptableMessage);
        push_if!(notification, EventKind::Notification);
        push_if!(chat_presence, EventKind::ChatPresence);
        push_if!(presence, EventKind::Presence);
        push_if!(picture_update, EventKind::PictureUpdate);
        push_if!(user_about_update, EventKind::UserAboutUpdate);
        push_if!(group_info_update, EventKind::GroupUpdate);
        push_if!(contact_update, EventKind::ContactUpdate);
        push_if!(self_push_name_updated, EventKind::SelfPushNameUpdated);
        push_if!(pin_update, EventKind::PinUpdate);
        push_if!(mute_update, EventKind::MuteUpdate);
        push_if!(archive_update, EventKind::ArchiveUpdate);
        push_if!(mark_chat_as_read_update, EventKind::MarkChatAsReadUpdate);
        push_if!(history_sync, EventKind::HistorySync);
        push_if!(offline_sync_preview, EventKind::OfflineSyncPreview);
        push_if!(offline_sync_completed, EventKind::OfflineSyncCompleted);
        push_if!(device_list_update, EventKind::DeviceListUpdate);
        push_if!(business_status_update, EventKind::BusinessStatusUpdate);
        push_if!(stream_replaced, EventKind::StreamReplaced);
        push_if!(temporary_ban, EventKind::TemporaryBan);
        push_if!(connect_failure, EventKind::ConnectFailure);
        push_if!(stream_error, EventKind::StreamError);
        push_if!(disappearing_mode_changed, EventKind::DisappearingModeChanged);
        push_if!(contact_number_changed, EventKind::ContactNumberChanged);
        push_if!(contact_sync_requested, EventKind::ContactSyncRequested);
        push_if!(contact_updated, EventKind::ContactUpdated);
        push_if!(star_update, EventKind::StarUpdate);
        push_if!(newsletter_live_update, EventKind::NewsletterLiveUpdate);
        push_if!(delete_chat_update, EventKind::DeleteChatUpdate);
        push_if!(delete_message_for_me_update, EventKind::DeleteMessageForMeUpdate);
        push_if!(incoming_call, EventKind::IncomingCall);
        push_if!(identity_change, EventKind::IdentityChange);
        push_if!(lock_chat_update, EventKind::LockChatUpdate);
        push_if!(favorites_update, EventKind::FavoritesUpdate);
        push_if!(status_privacy_update, EventKind::StatusPrivacyUpdate);
        push_if!(unarchive_chats_setting_update, EventKind::UnarchiveChatsSettingUpdate);
        push_if!(disable_link_previews_update, EventKind::DisableLinkPreviewsUpdate);
        push_if!(contact_removed, EventKind::ContactRemoved);
        push_if!(offline_sync_interrupted, EventKind::OfflineSyncInterrupted);
        push_if!(client_expiration_changed, EventKind::ClientExpirationChanged);
        push_if!(server_ack, EventKind::ServerAck);
        push_if!(app_state_sync_failed, EventKind::AppStateSyncFailed);
        push_if!(mex_notification, EventKind::MexNotification);
        kinds
    }
}
