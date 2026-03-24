use pyo3::prelude::*;

use self::clients::tryx_client::TryxClient;
use self::clients::tryx::Tryx;
use self::clients::chat_actions::ChatActionsClient;
use self::clients::community::CommunityClient;
use self::clients::contacts::ContactClient;
use self::clients::groups::GroupsClient;
use self::clients::newsletter::NewsletterClient;
use self::clients::status::StatusClient;
use self::helpers::groups::GroupsHelpers;
use self::helpers::newsletter::NewsletterHelpers;
use self::helpers::status::StatusHelpers;
use self::events::types::{
    BusinessStatusUpdateData,
    EvArchiveUpdateData,
    EvArchiveUpdate,
    EvBusinessStatusUpdate,
    EvChatPresence,
    EvClientOutDated,
    EvConnectFailure,
    EvContactNumberChangedData,
    EvContactNumberChanged,
    EvContactSyncRequestedData,
    EvContactSyncRequested,
    EvConnected,
    EvContactUpdatedData,
    EvContactUpdated,
    EvContactUpdate,
    EvDeviceListUpdate,
    EvDisconnected,
    EvDisappearingModeChangedData,
    EvDisappearingModeChanged,
    EvGroupInfoUpdate,
    EvGroupUpdate,
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
    EvStarUpdateData,
    EvStarUpdate,
    EvStreamError,
    EvStreamReplaced,
    EvTemporaryBan,
    EvUndecryptableMessage,
    EvUserAboutUpdate,
    GroupUpdateData,
    LazyConversation,
    MarkChatAsReadUpdateData,
    MessageData,
    MuteUpdateData,
    OfflineSyncCompletedData,
    OfflineSyncData,
    PairSuccessData,
    PictureUpdateData,
    PinUpdatedata,
    UserAboutUpdateData,
    DeviceListUpdateData,
    EvPushNameUpdateData,
};
use self::backend::SqliteBackend;
use self::exceptions::{EventDispatchError, FailedBuildBot, PyPayloadBuildError, UnsupportedBackend, UnsupportedEventType};
use self::types::{JID, MessageInfo, UploadResponse};
use self::wacore::download::MediaType;
use self::wacore::iq::usync::{ContactInfo, IsOnWhatsAppResult, UserInfo};
use self::wacore::iq::community::{
    CommunitySubgroup,
    CreateCommunityOptions,
    CreateCommunityResult,
    GroupMetadata,
    GroupParticipant,
    GroupType,
    LinkSubgroupsResult,
    UnlinkSubgroupsResult,
};
use self::wacore::iq::newsletter::{
    NewsletterMessage,
    NewsletterMetadata,
    NewsletterReactionCount,
    NewsletterRole,
    NewsletterState,
    NewsletterVerification,
};
use self::wacore::iq::groups::{
    CreateGroupOptions,
    CreateGroupResult,
    GroupInfo,
    GroupParticipantOptions,
    JoinGroupResult,
    MemberAddMode,
    MemberLinkMode,
    MembershipApprovalMode,
    MembershipRequest,
    ParticipantChangeResponse,
};
use self::wacore::iq::status::{StatusPrivacySetting, StatusSendOptions};

/// A Python module implemented in Rust.
/// 
#[pymodule]
fn _tryx(_py: &Bound<PyModule>) -> PyResult<()> {
    // m.
    let client_module = PyModule::new(_py.py(), "client")?;
    client_module.add_class::<TryxClient>()?;
    client_module.add_class::<ContactClient>()?;
    client_module.add_class::<ChatActionsClient>()?;
    client_module.add_class::<CommunityClient>()?;
    client_module.add_class::<NewsletterClient>()?;
    client_module.add_class::<GroupsClient>()?;
    client_module.add_class::<StatusClient>()?;
    client_module.add_class::<ContactInfo>()?;
    client_module.add_class::<IsOnWhatsAppResult>()?;
    client_module.add_class::<UserInfo>()?;
    client_module.add_class::<GroupType>()?;
    client_module.add_class::<CreateCommunityOptions>()?;
    client_module.add_class::<CreateCommunityResult>()?;
    client_module.add_class::<CommunitySubgroup>()?;
    client_module.add_class::<LinkSubgroupsResult>()?;
    client_module.add_class::<UnlinkSubgroupsResult>()?;
    client_module.add_class::<GroupParticipant>()?;
    client_module.add_class::<GroupMetadata>()?;
    client_module.add_class::<NewsletterVerification>()?;
    client_module.add_class::<NewsletterState>()?;
    client_module.add_class::<NewsletterRole>()?;
    client_module.add_class::<NewsletterReactionCount>()?;
    client_module.add_class::<NewsletterMetadata>()?;
    client_module.add_class::<NewsletterMessage>()?;
    client_module.add_class::<MemberLinkMode>()?;
    client_module.add_class::<MemberAddMode>()?;
    client_module.add_class::<MembershipApprovalMode>()?;
    client_module.add_class::<GroupParticipantOptions>()?;
    client_module.add_class::<CreateGroupOptions>()?;
    client_module.add_class::<CreateGroupResult>()?;
    client_module.add_class::<JoinGroupResult>()?;
    client_module.add_class::<ParticipantChangeResponse>()?;
    client_module.add_class::<MembershipRequest>()?;
    client_module.add_class::<GroupInfo>()?;
    client_module.add_class::<StatusPrivacySetting>()?;
    client_module.add_class::<StatusSendOptions>()?;
    client_module.add_class::<Tryx>()?;
    _py.add_submodule(&client_module)?;

    let events_module = PyModule::new(_py.py(), "events")?;
    events_module.add_class::<EvConnected>()?;
    events_module.add_class::<EvDisconnected>()?;
    events_module.add_class::<EvLoggedOut>()?;
    events_module.add_class::<EvPairingQrCode>()?;
    events_module.add_class::<EvPairingCode>()?;
    events_module.add_class::<EvPairError>()?;
    events_module.add_class::<PairSuccessData>()?;
    events_module.add_class::<EvPairSuccess>()?;
    events_module.add_class::<EvQrScannedWithoutMultidevice>()?;
    events_module.add_class::<EvClientOutDated>()?;
    events_module.add_class::<EvStreamReplaced>()?;
    events_module.add_class::<EvTemporaryBan>()?;
    events_module.add_class::<EvConnectFailure>()?;
    events_module.add_class::<EvStreamError>()?;
    events_module.add_class::<EvReceipt>()?;
    events_module.add_class::<EvUndecryptableMessage>()?;
    events_module.add_class::<MessageData>()?;
    events_module.add_class::<EvMessage>()?;
    events_module.add_class::<EvNotification>()?;
    events_module.add_class::<EvChatPresence>()?;
    events_module.add_class::<EvPresence>()?;
    events_module.add_class::<PictureUpdateData>()?;
    events_module.add_class::<EvPictureUpdate>()?;
    events_module.add_class::<UserAboutUpdateData>()?;
    events_module.add_class::<EvUserAboutUpdate>()?;
    events_module.add_class::<LazyConversation>()?;
    events_module.add_class::<EvJoinedGroup>()?;
    events_module.add_class::<EvGroupInfoUpdate>()?;
    events_module.add_class::<EvPushNameUpdateData>()?;
    events_module.add_class::<EvPushNameUpdate>()?;
    events_module.add_class::<EvSelfPushNameUpdated>()?;
    events_module.add_class::<PinUpdatedata>()?;
    events_module.add_class::<EvPinUpdate>()?;
    events_module.add_class::<MuteUpdateData>()?;
    events_module.add_class::<EvMuteUpdate>()?;
    events_module.add_class::<MarkChatAsReadUpdateData>()?;
    events_module.add_class::<EvMarkChatAsReadUpdate>()?;
    events_module.add_class::<EvHistorySync>()?;
    events_module.add_class::<OfflineSyncData>()?;
    events_module.add_class::<EvOfflineSyncPreview>()?;
    events_module.add_class::<OfflineSyncCompletedData>()?;
    events_module.add_class::<EvOfflineSyncCompleted>()?;
    events_module.add_class::<DeviceListUpdateData>()?;
    events_module.add_class::<EvDeviceListUpdate>()?;
    events_module.add_class::<BusinessStatusUpdateData>()?;
    events_module.add_class::<EvBusinessStatusUpdate>()?;
    events_module.add_class::<EvArchiveUpdateData>()?;
    events_module.add_class::<EvArchiveUpdate>()?;
    events_module.add_class::<EvDisappearingModeChangedData>()?;
    events_module.add_class::<EvDisappearingModeChanged>()?;
    events_module.add_class::<EvContactNumberChangedData>()?;
    events_module.add_class::<EvContactNumberChanged>()?;
    events_module.add_class::<EvContactSyncRequestedData>()?;
    events_module.add_class::<EvContactSyncRequested>()?;
    events_module.add_class::<EvContactUpdatedData>()?;
    events_module.add_class::<EvContactUpdated>()?;
    events_module.add_class::<EvStarUpdateData>()?;
    events_module.add_class::<EvStarUpdate>()?;
    events_module.add_class::<GroupUpdateData>()?;
    events_module.add_class::<EvGroupUpdate>()?;
    events_module.add_class::<EvContactUpdate>()?;
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

    let helpers_module = PyModule::new(_py.py(), "helpers")?;
    helpers_module.add_class::<NewsletterHelpers>()?;
    helpers_module.add_class::<GroupsHelpers>()?;
    helpers_module.add_class::<StatusHelpers>()?;
    _py.add_submodule(&helpers_module)?;
    Ok(())
}

mod backend;
mod clients;
mod events;
mod types;
mod exceptions;
mod wacore;
mod log;
mod helpers;