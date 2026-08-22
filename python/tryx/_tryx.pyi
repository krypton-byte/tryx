"""
Type stubs for the ``_tryx`` compiled Rust extension.

Hand-crafted for accuracy — every type reflects the actual Rust source.
Generated from Rust source analysis; do not edit manually without
cross-checking against ``src/`` changes.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, ClassVar, Coroutine

from .waproto.whatsapp_pb2 import (
    Conversation as _Conversation,
    HistorySync as _HistorySync,
    Message as _Message,
    MessageKey as _MessageKey,
    SyncActionValue as _SyncActionValue,
)

# Nested types inside SyncActionValue
_ContactAction = _SyncActionValue.ContactAction
_DeleteChatAction = _SyncActionValue.DeleteChatAction
_DeleteMessageForMeAction = _SyncActionValue.DeleteMessageForMeAction
_MarkChatAsReadAction = _SyncActionValue.MarkChatAsReadAction
_MuteAction = _SyncActionValue.MuteAction
_SyncActionMessageRange = _SyncActionValue.SyncActionMessageRange

_T = Coroutine[Any, Any, Any]

# ═══════════════════════════════════════════════════════════════════
#  _tryx.types
# ═══════════════════════════════════════════════════════════════════

class JID:
    """WhatsApp JID (user + server pair)."""
    @property
    def user(self) -> str: ...
    @property
    def server(self) -> str: ...
    def __init__(self, user: str, server: str) -> None: ...
    def __repr__(self) -> str: ...

class MessageSource:
    """Origin and routing metadata for a message."""
    @property
    def chat(self) -> JID: ...
    @property
    def sender(self) -> JID: ...
    @property
    def is_from_me(self) -> bool: ...
    @property
    def is_group(self) -> bool: ...
    @property
    def addressing_mode(self) -> str | None: ...
    @property
    def sender_alt(self) -> JID | None: ...
    @property
    def recipient_alt(self) -> JID | None: ...
    @property
    def broadcast_list_owner(self) -> JID | None: ...
    @property
    def recipient(self) -> JID | None: ...

class MsgBotInfo:
    """Bot metadata attached to a message."""
    edit_target_id: str | None
    @property
    def edit_type(self) -> str | None: ...
    @property
    def edit_sender_timestamp(self) -> datetime | None: ...

class MsgMetaInfo:
    """Thread / reply metadata for a message."""
    target_id: str | None
    target_sender: JID | None
    target_chat: JID | None
    thread_message_id: str | None
    thread_message_sender_jid: JID | None
    content_type: str | None
    appdata: str | None
    reporting_tag: bytes | None
    reporting_token: bytes | None
    reporting_token_version: int | None

class DeviceSentMeta:
    """Device-sent metadata."""
    destination_jid: str
    phash: str

class MessageInfo:
    """Normalized metadata for a received or sent message."""
    id: str
    type: str
    push_name: str
    @property
    def source(self) -> MessageSource: ...
    @property
    def multicast(self) -> bool: ...
    @property
    def server_id(self) -> int: ...
    @property
    def timestamp(self) -> datetime: ...
    @property
    def media_type(self) -> str | None: ...
    @property
    def edit(self) -> str: ...
    @property
    def bot_info(self) -> MsgBotInfo | None: ...
    @property
    def meta_info(self) -> MsgMetaInfo: ...
    @property
    def verified_name(self) -> Any | None: ...
    @property
    def device_sent_meta(self) -> DeviceSentMeta | None: ...
    @property
    def category(self) -> str: ...
    @property
    def ephemeral_expiration(self) -> int | None: ...
    @property
    def is_offline(self) -> bool: ...
    @property
    def unavailable_request_id(self) -> str | None: ...
    @property
    def server_timestamp_us(self) -> int | None: ...
    @property
    def verified_level(self) -> str | None: ...
    @property
    def verified_name_serial(self) -> int | None: ...
    @property
    def peer_recipient_pn(self) -> JID | None: ...
    @property
    def bcl_participants(self) -> list[JID]: ...

class ProfilePicture:
    """Profile picture download result."""
    id: str
    url: str
    direct_path: str | None
    hash: str | None

class SendResult:
    """Result of a successful message send operation."""
    message_id: str
    to: JID

class UploadResponse:
    """Media upload response from WhatsApp servers."""
    @property
    def url(self) -> str: ...
    @property
    def direct_path(self) -> str: ...
    @property
    def media_key(self) -> bytes: ...
    @property
    def file_enc_sha256(self) -> bytes: ...
    @property
    def file_sha256(self) -> bytes: ...
    @property
    def file_length(self) -> int: ...
    @property
    def media_key_timestamp(self) -> int: ...
    @property
    def streaming_sidecar(self) -> bytes | None: ...

class MediaReuploadResult:
    """Result of a media reupload request."""
    status: str
    direct_path: str | None

# ═══════════════════════════════════════════════════════════════════
#  _tryx.backend
# ═══════════════════════════════════════════════════════════════════

class BackendBase:
    """Base class for storage backends."""
    pass

class SqliteStore:
    """SQLite-backed storage for WhatsApp session data."""
    path: str
    def __init__(self, path: str) -> None: ...

# ═══════════════════════════════════════════════════════════════════
#  _tryx.wacore  (IQ types, enums, data classes)
# ═══════════════════════════════════════════════════════════════════

# ── Enums ──

class ChatStateType:
    Composing: ClassVar[ChatStateType]
    Recording: ClassVar[ChatStateType]
    Paused: ClassVar[ChatStateType]

class PresenceStatus:
    Available: ClassVar[PresenceStatus]
    Unavailable: ClassVar[PresenceStatus]

class MediaType:
    Image: ClassVar[MediaType]
    Video: ClassVar[MediaType]
    Audio: ClassVar[MediaType]
    Document: ClassVar[MediaType]
    History: ClassVar[MediaType]
    AppState: ClassVar[MediaType]
    Sticker: ClassVar[MediaType]
    StickerPack: ClassVar[MediaType]
    LinkThumbnail: ClassVar[MediaType]

class MemberLinkMode:
    AdminLink: ClassVar[MemberLinkMode]
    AllMemberLink: ClassVar[MemberLinkMode]

class MemberAddMode:
    AdminAdd: ClassVar[MemberAddMode]
    AllMemberAdd: ClassVar[MemberAddMode]

class MembershipApprovalMode:
    Off: ClassVar[MembershipApprovalMode]
    On: ClassVar[MembershipApprovalMode]

class GroupType:
    Default: ClassVar[GroupType]
    Community: ClassVar[GroupType]
    LinkedSubgroup: ClassVar[GroupType]
    LinkedAnnouncementGroup: ClassVar[GroupType]
    LinkedGeneralGroup: ClassVar[GroupType]

class NewsletterRole:
    Owner: ClassVar[NewsletterRole]
    Admin: ClassVar[NewsletterRole]
    Subscriber: ClassVar[NewsletterRole]
    Guest: ClassVar[NewsletterRole]

class NewsletterState:
    Active: ClassVar[NewsletterState]
    Suspended: ClassVar[NewsletterState]
    Geosuspended: ClassVar[NewsletterState]

class NewsletterVerification:
    Verified: ClassVar[NewsletterVerification]
    Unverified: ClassVar[NewsletterVerification]

class PrivacyCategory:
    Last: ClassVar[PrivacyCategory]
    Online: ClassVar[PrivacyCategory]
    Profile: ClassVar[PrivacyCategory]
    Status: ClassVar[PrivacyCategory]
    GroupAdd: ClassVar[PrivacyCategory]
    ReadReceipts: ClassVar[PrivacyCategory]
    CallAdd: ClassVar[PrivacyCategory]
    Messages: ClassVar[PrivacyCategory]
    DefenseMode: ClassVar[PrivacyCategory]
    Other: ClassVar[PrivacyCategory]

class PrivacyValue:
    All: ClassVar[PrivacyValue]
    Contacts: ClassVar[PrivacyValue]
    None_: ClassVar[PrivacyValue]
    ContactBlacklist: ClassVar[PrivacyValue]
    MatchLastSeen: ClassVar[PrivacyValue]
    Known: ClassVar[PrivacyValue]
    Off: ClassVar[PrivacyValue]
    OnStandard: ClassVar[PrivacyValue]
    Other: ClassVar[PrivacyValue]

class StatusPrivacySetting:
    Contacts: ClassVar[StatusPrivacySetting]
    AllowList: ClassVar[StatusPrivacySetting]
    DenyList: ClassVar[StatusPrivacySetting]

class DisallowedListAction:
    Add: ClassVar[DisallowedListAction]
    Remove: ClassVar[DisallowedListAction]

# ── IQ Data Classes ──

class GroupParticipant:
    jid: JID
    phone_number: JID | None
    is_admin: bool

class GroupParticipantOptions:
    jid: JID
    phone_number: JID | None
    privacy: bytes | None
    def __init__(self, jid: JID, phone_number: JID | None = None, privacy: bytes | None = None) -> None: ...

class CreateGroupOptions:
    subject: str
    participants: list[GroupParticipantOptions]
    member_link_mode: MemberLinkMode | None
    member_add_mode: MemberAddMode | None
    membership_approval_mode: MembershipApprovalMode | None
    ephemeral_expiration: int | None
    is_parent: bool
    closed: bool
    allow_non_admin_sub_group_creation: bool
    create_general_chat: bool
    def __init__(self, subject: str, participants: list[GroupParticipantOptions] = ..., member_link_mode: MemberLinkMode | None = ..., member_add_mode: MemberAddMode | None = ..., membership_approval_mode: MembershipApprovalMode | None = ..., ephemeral_expiration: int | None = ..., is_parent: bool = False, closed: bool = False, allow_non_admin_sub_group_creation: bool = False, create_general_chat: bool = False) -> None: ...

class GroupMetadata:
    id: JID
    subject: str
    participants: list[GroupParticipant]
    addressing_mode: str
    creator: JID | None
    creation_time: int | None
    subject_time: int | None
    subject_owner: JID | None
    description: str | None
    description_id: str | None
    is_locked: bool
    is_announcement: bool
    ephemeral_expiration: int
    membership_approval: bool
    member_add_mode: str | None
    member_link_mode: str | None
    size: int | None
    is_parent_group: bool
    parent_group_jid: JID | None
    is_default_sub_group: bool
    is_general_chat: bool
    allow_non_admin_sub_group_creation: bool
    @property
    def group_type(self) -> GroupType: ...

class CreateGroupResult:
    gid: JID
    metadata: GroupMetadata

class JoinGroupResult:
    jid: JID
    pending_approval: bool

class ParticipantChangeResponse:
    jid: JID
    status: str | None
    error: str | None

class MembershipRequest:
    jid: JID
    request_time: int | None

class GroupInfo:
    participants: list[JID]
    addressing_mode: str
    lid_to_pn_map: list[tuple[str, JID]]

class IsOnWhatsAppResult:
    jid: JID
    is_registered: bool

class ContactInfo:
    jid: JID
    lid: JID | None
    is_registered: bool
    is_business: bool
    status: str | None
    picture_id: int | None

class UserInfo:
    jid: JID
    lid: JID | None
    status: str | None
    picture_id: str | None
    is_business: bool

class BlocklistEntry:
    jid: JID
    timestamp: int | None

class CreateCommunityOptions:
    name: str
    description: str | None
    closed: bool
    allow_non_admin_sub_group_creation: bool
    create_general_chat: bool
    def __init__(self, name: str, description: str | None = None, closed: bool = False, allow_non_admin_sub_group_creation: bool = False, create_general_chat: bool = True) -> None: ...

class CreateCommunityResult:
    gid: JID
    metadata: GroupMetadata

class CommunitySubgroup:
    id: JID
    subject: str
    participant_count: int | None
    is_default_sub_group: bool
    is_general_chat: bool

class LinkSubgroupsResult:
    linked_jids: list[JID]
    failed_groups: list[tuple[JID, int]]

class UnlinkSubgroupsResult:
    unlinked_jids: list[JID]
    failed_groups: list[tuple[JID, int]]

class PollOptionResult:
    name: str
    voters: list[str]

class PrivacySetting:
    category: PrivacyCategory
    value: PrivacyValue

class DisallowedListUserEntry:
    action: DisallowedListAction
    jid: JID
    pn_jid: JID | None
    def __init__(self, action: DisallowedListAction, jid: JID, pn_jid: JID | None = None) -> None: ...

class DisallowedListUpdate:
    dhash: str
    users: list[DisallowedListUserEntry]
    def __init__(self, dhash: str, users: list[DisallowedListUserEntry] = ...) -> None: ...

class StatusSendOptions:
    privacy: StatusPrivacySetting
    def __init__(self, privacy: StatusPrivacySetting = ...) -> None: ...

class NewsletterMetadata:
    jid: JID
    name: str
    description: str | None
    subscriber_count: int
    verification: NewsletterVerification
    state: NewsletterState
    picture_url: str | None
    preview_url: str | None
    invite_code: str | None
    role: NewsletterRole | None
    creation_time: int | None

class NewsletterAdminProfile:
    id: str | None
    name: str
    picture_id: str | None
    picture_direct_path: str | None

class NewsletterAdminInfo:
    admin_count: int | None
    admin_profile: NewsletterAdminProfile | None
    admin_profiles_enabled: bool | None

class NewsletterFollower:
    jid: JID
    phone_jid: JID | None
    display_name: str | None
    username: str | None
    role: NewsletterRole | None
    follow_time: int | None
    admin_profile: NewsletterAdminProfile | None

class NewsletterReactionCount:
    code: str
    count: int

class NewsletterMessage:
    server_id: int
    timestamp: int
    message_type: str
    is_sender: bool
    reactions: list[NewsletterReactionCount]
    @property
    def message(self) -> Any | None: ...

# ── WACore Node types ──

class NodeValue:
    @property
    def value(self) -> str | JID: ...
    @value.setter
    def value(self, value: str | JID) -> None: ...
    def __init__(self, value: str) -> None: ...
    @staticmethod
    def jid(value: JID) -> NodeValue: ...
    def set_string(self, value: str) -> None: ...
    def set_jid(self, value: JID) -> None: ...

class NodeContent:
    @property
    def value(self) -> bytes | str | list[Node]: ...
    def is_bytes(self) -> bool: ...
    def is_string(self) -> bool: ...
    def is_nodes(self) -> bool: ...

class Attrs:
    key: str
    value: NodeValue
    def __init__(self, key: str, value: NodeValue) -> None: ...

class Node:
    tag: str
    attrs: list[Attrs]
    content: NodeContent | None
    def __init__(self, tag: str, attrs: list[Attrs], content: NodeContent | None = None) -> None: ...

class KeyIndexInfo:
    timestamp: int
    signed_bytes: bytes | None

class BusinessSubscription:
    id: str
    status: str
    expiration_date: datetime | None
    creation_time: datetime | None

class DeviceProp:
    name: str
    manufacturer: str
    model: str
    os_version: str

# ═══════════════════════════════════════════════════════════════════
#  _tryx.events  (Event classes & data payloads)
# ═══════════════════════════════════════════════════════════════════

# ── Lifecycle Events ──

class EvConnected:
    pass

class EvDisconnected:
    pass

class EvLoggedOut:
    on_connect: bool
    reason: str

class EvStreamReplaced:
    pass

class EvClientOutDated:
    pass

class EvQrScannedWithoutMultidevice:
    pass

# ── Pairing Events ──

class EvPairingQrCode:
    code: str
    timeout: int

class EvPairingCode:
    code: str
    timeout: int

class PairSuccessData:
    id: JID
    lid: JID
    business_name: str
    platform: str

class EvPairSuccess:
    @property
    def data(self) -> PairSuccessData: ...

class EvPairError:
    id: JID
    lid: JID
    business_name: str
    platform: str
    error: str

# ── Presence / Chat Presence ──

class EvPresence:
    from_: JID
    unavailable: bool
    last_seen: datetime | None

class EvChatPresence:
    @property
    def source(self) -> MessageSource: ...
    @property
    def state(self) -> str: ...
    @property
    def media(self) -> str: ...

# ── Messaging Events ──

class MessageData:
    """Parsed message content with convenience accessors."""
    @property
    def conversation(self) -> str | None: ...
    @property
    def caption(self) -> str | None: ...
    @property
    def message_info(self) -> MessageInfo: ...
    @property
    def raw_proto(self) -> _Message: ...
    def get_extended_text_message(self) -> str | None: ...
    def get_text(self) -> str | None: ...

class EvMessage:
    @property
    def data(self) -> MessageData: ...

class EvNotification:
    @property
    def node(self) -> Node: ...

class EvReceipt:
    message_ids: list[str]
    timestamp: datetime
    receipt_type: ReceiptType
    @property
    def source(self) -> MessageSource | None: ...

class EvUndecryptableMessage:
    is_unavailable: bool
    unavailable_type: UnavailableType
    decrypt_fail_mode: DecryptFailMode
    @property
    def info(self) -> MessageInfo | None: ...

class EvHistorySync:
    @property
    def proto(self) -> _HistorySync: ...

# ── Receipt / Decrypt Enums ──

class ReceiptType:
    Delivered: ClassVar[ReceiptType]
    Sender: ClassVar[ReceiptType]
    Retry: ClassVar[ReceiptType]
    Read: ClassVar[ReceiptType]
    ReadSelf: ClassVar[ReceiptType]
    Played: ClassVar[ReceiptType]
    PlayedSelf: ClassVar[ReceiptType]
    ServerError: ClassVar[ReceiptType]
    Inactive: ClassVar[ReceiptType]
    PeerMsg: ClassVar[ReceiptType]
    HistorySync: ClassVar[ReceiptType]
    EncRekeyRetry: ClassVar[ReceiptType]
    Other: ClassVar[ReceiptType]

class UnavailableType:
    Unknown: ClassVar[UnavailableType]
    ViewOnce: ClassVar[UnavailableType]
    Hosted: ClassVar[UnavailableType]
    Bot: ClassVar[UnavailableType]

class DecryptFailMode:
    Show: ClassVar[DecryptFailMode]
    Hide: ClassVar[DecryptFailMode]

# ── Sync Action Events ──

class PinUpdatedata:
    jid: JID
    timestamp: datetime
    pinned: bool | None
    from_full_sync: bool

class EvPinUpdate:
    @property
    def data(self) -> PinUpdatedata: ...

class MuteUpdateData:
    jid: JID
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> _MuteAction: ...

class EvMuteUpdate:
    @property
    def data(self) -> MuteUpdateData: ...

class EvArchiveUpdateData:
    jid: JID
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> _SyncActionValue: ...

class EvArchiveUpdate:
    @property
    def data(self) -> EvArchiveUpdateData: ...

class MarkChatAsReadUpdateData:
    jid: JID
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> _MarkChatAsReadAction: ...

class EvMarkChatAsReadUpdate:
    @property
    def data(self) -> MarkChatAsReadUpdateData: ...

class EvStarUpdateData:
    chat_jid: JID
    participant_jid: JID | None
    message_id: str
    from_me: bool
    timestamp: datetime
    from_full_sync: bool
    starred: bool | None

class EvStarUpdate:
    @property
    def data(self) -> EvStarUpdateData: ...

class EvDisappearingModeChangedData:
    from_: JID
    duration: int
    setting_timestamp: int

class EvDisappearingModeChanged:
    @property
    def data(self) -> EvDisappearingModeChangedData: ...

class OfflineSyncData:
    total: int
    app_data_changes: int
    messages: int
    notifications: int
    receipts: int

class EvOfflineSyncPreview:
    @property
    def data(self) -> OfflineSyncData: ...

class OfflineSyncCompletedData:
    count: int

class EvOfflineSyncCompleted:
    @property
    def data(self) -> OfflineSyncCompletedData: ...

# ── Contact Events ──

class ContactUpdateData:
    jid: JID
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> _ContactAction: ...

class EvContactUpdate:
    @property
    def data(self) -> ContactUpdateData: ...

class EvContactUpdatedData:
    jid: JID
    timestamp: datetime

class EvContactUpdated:
    @property
    def data(self) -> EvContactUpdatedData: ...

class EvContactNumberChangedData:
    old_jid: JID
    new_jid: JID
    old_lid: JID | None
    new_lid: JID | None
    timestamp: datetime

class EvContactNumberChanged:
    @property
    def data(self) -> EvContactNumberChangedData: ...

class EvContactSyncRequestedData:
    after: datetime | None
    timestamp: datetime

class EvContactSyncRequested:
    @property
    def data(self) -> EvContactSyncRequestedData: ...

# ── Push Name Events ──

class EvPushNameUpdateData:
    jid: JID | None
    message: MessageInfo | None
    old_push_name: str
    new_push_name: str

class EvPushNameUpdate:
    @property
    def data(self) -> EvPushNameUpdateData: ...

class EvSelfPushNameUpdated:
    from_server: bool
    old_name: str
    new_name: str

# ── Picture Events ──

class PictureUpdateData:
    jid: JID
    author: JID | None
    timestamp: datetime | None
    removed: bool
    picture_id: str | None

class EvPictureUpdate:
    @property
    def data(self) -> PictureUpdateData: ...

# ── User About Events ──

class UserAboutUpdateData:
    jid: JID
    status: str
    timestamp: datetime | None

class EvUserAboutUpdate:
    @property
    def data(self) -> UserAboutUpdateData: ...

# ── Group Events ──

class GroupParticipantEvents:
    """Participant in a group notification action (events module)."""
    jid: JID
    phone_number: JID | None

class GroupNotificationAction:
    """Discriminated union of group notification actions."""
    @staticmethod
    def Add(participants: list[GroupParticipantEvents], reason: str | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def Remove(participants: list[GroupParticipantEvents], reason: str | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def Promote(participants: list[GroupParticipantEvents]) -> GroupNotificationAction: ...
    @staticmethod
    def Demote(participants: list[GroupParticipantEvents]) -> GroupNotificationAction: ...
    @staticmethod
    def Modify(participants: list[GroupParticipantEvents]) -> GroupNotificationAction: ...
    @staticmethod
    def Subject(subject: str, subject_owner: JID | None = None, subject_timestamp: datetime | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def Description(id: str, description: str | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def Locked(threshold: str | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def Unlocked() -> GroupNotificationAction: ...
    @staticmethod
    def Announce() -> GroupNotificationAction: ...
    @staticmethod
    def NotAnnounce() -> GroupNotificationAction: ...
    @staticmethod
    def Ephemeral(expiration: int, trigger: int | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def MembershipApprovalMode(enabled: bool) -> GroupNotificationAction: ...
    @staticmethod
    def MembershipApprovalRequest(request_method: str, parent_group_jid: JID | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def CreatedMembershipRequests(request_method: str, parent_group_jid: JID | None = None, requests: list[GroupParticipantEvents] = ...) -> GroupNotificationAction: ...
    @staticmethod
    def RevokedMembershipRequests(participants: list[JID]) -> GroupNotificationAction: ...
    @staticmethod
    def MemberAddMode(mode: str) -> GroupNotificationAction: ...
    @staticmethod
    def NoFrequentlyForwarded() -> GroupNotificationAction: ...
    @staticmethod
    def FrequentlyForwardedOk() -> GroupNotificationAction: ...
    @staticmethod
    def Invite(code: str) -> GroupNotificationAction: ...
    @staticmethod
    def RevokeInvite() -> GroupNotificationAction: ...
    @staticmethod
    def GrowthLocked(expiration: int, lock_type: str) -> GroupNotificationAction: ...
    @staticmethod
    def GrowthUnlocked() -> GroupNotificationAction: ...
    @staticmethod
    def Create(raw: Node) -> GroupNotificationAction: ...
    @staticmethod
    def Delete(reason: str | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def Link(link_type: str, raw: Node) -> GroupNotificationAction: ...
    @staticmethod
    def Unlink(unlink_type: str, unlink_reason: str | None = None, raw: Node = ...) -> GroupNotificationAction: ...
    @staticmethod
    def LinkedGroupPromote(participants: list[GroupParticipantEvents]) -> GroupNotificationAction: ...
    @staticmethod
    def LinkedGroupDemote(participants: list[GroupParticipantEvents]) -> GroupNotificationAction: ...
    @staticmethod
    def Suspended() -> GroupNotificationAction: ...
    @staticmethod
    def Unsuspended() -> GroupNotificationAction: ...
    @staticmethod
    def AutoAddDisabled() -> GroupNotificationAction: ...
    @staticmethod
    def IsCapiHostedGroup() -> GroupNotificationAction: ...
    @staticmethod
    def GroupSafetyCheck() -> GroupNotificationAction: ...
    @staticmethod
    def LimitSharingEnabled(trigger: int | None = None) -> GroupNotificationAction: ...
    @staticmethod
    def AllowAdminReports() -> GroupNotificationAction: ...
    @staticmethod
    def NotAllowAdminReports() -> GroupNotificationAction: ...
    @staticmethod
    def Reports() -> GroupNotificationAction: ...
    @staticmethod
    def AllowNonAdminSubGroupCreation() -> GroupNotificationAction: ...
    @staticmethod
    def NotAllowNonAdminSubGroupCreation() -> GroupNotificationAction: ...
    @staticmethod
    def CreatedSubGroupSuggestion(raw: Node) -> GroupNotificationAction: ...
    @staticmethod
    def RevokedSubGroupSuggestions(raw: Node) -> GroupNotificationAction: ...
    @staticmethod
    def ChangeNumber(new_owner: JID | None = None, sub_group_suggestions: list[JID] = ...) -> GroupNotificationAction: ...
    @staticmethod
    def Unknown(tag: str) -> GroupNotificationAction: ...

class GroupUpdateData:
    group_jid: JID
    participant: JID | None
    participant_pn: JID | None
    timestamp: datetime
    is_lid_addressing_mode: bool
    action: GroupNotificationAction

class EvGroupUpdate:
    @property
    def data(self) -> GroupUpdateData: ...

# ── Newsletter Events ──

class NewsletterLiveUpdateReaction:
    code: str
    count: int

class NewsletterUpdateMessage:
    server_id: int
    reactions: list[NewsletterLiveUpdateReaction]

class NewsletterLiveUpdateData:
    newsletter_jid: JID
    messages: list[NewsletterUpdateMessage]

class EvNewsletterLiveUpdate:
    @property
    def data(self) -> NewsletterLiveUpdateData: ...

# ── Device List Events ──

class DeviceListUpdateType:
    Added: ClassVar[DeviceListUpdateType]
    Removed: ClassVar[DeviceListUpdateType]
    Updated: ClassVar[DeviceListUpdateType]

class DeviceNottificationInfo:
    device_id: int
    key_index: int | None

class DeviceListUpdateData:
    user: JID
    lid_user: JID | None
    update_type: DeviceListUpdateType
    devices: list[DeviceNottificationInfo]
    key_index: KeyIndexInfo | None
    contact_hash: str | None

class EvDeviceListUpdate:
    @property
    def data(self) -> DeviceListUpdateData: ...

# ── Business Status Events ──

class BusinessStatusUpdateType:
    RemovedAsBusiness: ClassVar[BusinessStatusUpdateType]
    VerifiedNameChanged: ClassVar[BusinessStatusUpdateType]
    ProfileUpdated: ClassVar[BusinessStatusUpdateType]
    ProductsUpdated: ClassVar[BusinessStatusUpdateType]
    CollectionsUpdated: ClassVar[BusinessStatusUpdateType]
    SubscriptionsUpdated: ClassVar[BusinessStatusUpdateType]
    Unknown: ClassVar[BusinessStatusUpdateType]

class BusinessStatusUpdateData:
    jid: JID
    update_type: BusinessStatusUpdateType
    timestamp: datetime
    target_jid: JID | None
    hash: str | None
    product_ids: list[str]
    collection_ids: list[str]
    subscriptions: list[BusinessSubscription]

class EvBusinessStatusUpdate:
    @property
    def data(self) -> BusinessStatusUpdateData: ...

# ── Error / Ban Events ──

class TempBanReason:
    SentToTooManyPeople: ClassVar[TempBanReason]
    SentBlockedNyUser: ClassVar[TempBanReason]
    CreateTooManyGroups: ClassVar[TempBanReason]
    SentTooManySameMessage: ClassVar[TempBanReason]
    Unknown: ClassVar[TempBanReason]

class EvTemporaryData:
    code: TempBanReason
    expire: datetime

class EvTemporaryBan:
    @property
    def data(self) -> EvTemporaryData: ...

class EvConnectFailure:
    reason: str
    message: str | None
    @property
    def node(self) -> Node | None: ...

class EvStreamError:
    code: str
    @property
    def node(self) -> Node | None: ...

# ── Lazy Conversation ──

class LazyConversation:
    @property
    def conversation(self) -> _Conversation | None: ...

class EvJoinedGroup:
    @property
    def data(self) -> LazyConversation: ...

# ── Delete Events ──

class DeleteChatUpdateData:
    jid: JID
    delete_media: bool
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> _DeleteChatAction: ...

class EvDeleteChatUpdate:
    @property
    def data(self) -> DeleteChatUpdateData: ...

class DeleteMessageForMeUpdateData:
    chat_jid: JID
    participant_jid: JID | None
    message_id: str
    from_me: bool
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> _DeleteMessageForMeAction: ...

class EvDeleteMessageForMeUpdate:
    @property
    def data(self) -> DeleteMessageForMeUpdateData: ...

# ── Dispatcher ──

class Dispatcher:
    def __init__(self) -> None: ...
    def on(self, event_type: type) -> Any: ...

# ═══════════════════════════════════════════════════════════════════
#  _tryx.exceptions
# ═══════════════════════════════════════════════════════════════════

class EventDispatchError(Exception): ...
class FailedBuildClient(Exception): ...
class FailedToDecodeProto(Exception): ...
class PyPayloadBuildError(Exception): ...
class UnsupportedBackend(Exception): ...
class UnsupportedEventType(Exception): ...

# ═══════════════════════════════════════════════════════════════════
#  _tryx.helpers
# ═══════════════════════════════════════════════════════════════════

class BlockingHelpers:
    @staticmethod
    def same_user(a: JID, b: JID) -> bool: ...

class ChatstateHelpers:
    @staticmethod
    def composing() -> ChatStateType: ...
    @staticmethod
    def recording() -> ChatStateType: ...
    @staticmethod
    def paused() -> ChatStateType: ...

class GroupsHelpers:
    @staticmethod
    def strip_invite_url(code: str) -> str: ...
    @staticmethod
    def build_participant(jid: JID, phone_number: JID | None = None, privacy: bytes | None = None) -> GroupParticipantOptions: ...
    @staticmethod
    def build_create_options(subject: str, participants: list[GroupParticipantOptions] = ..., member_link_mode: MemberLinkMode | None = ..., member_add_mode: MemberAddMode | None = ..., membership_approval_mode: MembershipApprovalMode | None = ..., ephemeral_expiration: int | None = ..., is_parent: bool = False, closed: bool = False, allow_non_admin_sub_group_creation: bool = False, create_general_chat: bool = False) -> CreateGroupOptions: ...

class NewsletterHelpers:
    @staticmethod
    def parse_message(data: bytes) -> _Message: ...
    @staticmethod
    def serialize_message(message: _Message) -> bytes: ...
    @staticmethod
    def build_text_message(text: str) -> _Message: ...

class PollsHelpers:
    @staticmethod
    def decrypt_vote(enc_payload: bytes, enc_iv: bytes, message_secret: bytes, poll_msg_id: str, poll_creator_jid: JID, voter_jid: JID) -> list[bytes]: ...
    @staticmethod
    def aggregate_votes(poll_options: list[str], votes: list[tuple[JID, bytes, bytes]], message_secret: bytes, poll_msg_id: str, poll_creator_jid: JID) -> list[PollOptionResult]: ...

class PresenceHelpers:
    @staticmethod
    def default_status() -> PresenceStatus: ...

class StatusHelpers:
    @staticmethod
    def build_send_options(privacy: StatusPrivacySetting = ...) -> StatusSendOptions: ...
    @staticmethod
    def default_privacy() -> StatusPrivacySetting: ...

# ═══════════════════════════════════════════════════════════════════
#  _tryx.client  (Client namespace classes)
# ═══════════════════════════════════════════════════════════════════

class Tryx:
    """Main entry point for WhatsApp automation."""
    handlers: Dispatcher
    def __init__(self, backend: Any) -> None: ...
    def get_client(self) -> TryxClient: ...
    def on(self, event_type: type) -> Any: ...
    async def run(self) -> None: ...
    def run_blocking(self) -> None: ...

class TryxClient:
    """Unified client exposing all namespace sub-clients."""
    contact: ContactClient
    chat_actions: ChatActionsClient
    community: CommunityClient
    newsletter: NewsletterClient
    groups: GroupsClient
    status: StatusClient
    chatstate: ChatstateClient
    blocking: BlockingClient
    polls: PollsClient
    presence: PresenceClient
    privacy: PrivacyClient
    profile: ProfileClient
    advanced: AdvancedClient
    labels: LabelsClient
    comments: CommentsClient
    events: EventsClient
    voip: VoipClient
    def is_connected(self) -> bool: ...
    async def download_media(self, message: Any) -> bytes: ...
    async def upload_file(self, path: str, media_type: MediaType) -> UploadResponse: ...
    async def upload(self, data: bytes, media_type: MediaType) -> UploadResponse: ...
    async def send_message(self, to: JID, message: _Message) -> SendResult: ...
    async def send_text(self, to: JID, text: str, quoted: EvMessage | None = None) -> SendResult: ...
    async def send_photo(self, to: JID, photo_data: bytes, mimetype: str | None = None, caption: str | None = None, quoted: EvMessage | None = None) -> SendResult: ...
    async def send_document(self, to: JID, document_data: bytes, mimetype: str | None = None, file_name: str | None = None, caption: str | None = None, quoted: EvMessage | None = None) -> SendResult: ...
    async def send_audio(self, to: JID, audio_data: bytes, mimetype: str | None = None, ptt: bool = False, seconds: int | None = None, quoted: EvMessage | None = None) -> SendResult: ...
    async def send_video(self, to: JID, video_data: bytes, mimetype: str | None = None, caption: str | None = None, seconds: int | None = None, gif_playback: bool = False, quoted: EvMessage | None = None) -> SendResult: ...
    async def send_gif(self, to: JID, gif_data: bytes, caption: str | None = None, seconds: int | None = None, quoted: EvMessage | None = None) -> SendResult: ...
    async def send_sticker(self, to: JID, sticker_data: bytes, is_animated: bool = False, quoted: EvMessage | None = None) -> SendResult: ...
    async def request_media_reupload(self, message_id: str, chat_jid: JID, media_key: bytes, is_from_me: bool = False, participant: JID | None = None) -> MediaReuploadResult: ...

# ── AdvancedClient ──

class AdvancedClient:
    def is_logged_in(self) -> bool: ...
    def get_push_name(self) -> str: ...
    def get_pn(self) -> JID | None: ...
    def get_lid(self) -> JID | None: ...
    async def stats(self) -> dict[str, Any]: ...
    async def memory_report_text(self) -> str: ...
    async def resource_report_text(self) -> str: ...
    async def wait_for_socket(self, timeout_seconds: float) -> None: ...
    async def wait_for_connected(self, timeout_seconds: float) -> None: ...
    async def wait_for_startup_sync(self, timeout_seconds: float) -> None: ...
    async def flush_pending_signal_state(self) -> None: ...
    async def send_raw_bytes(self, plaintext: bytes) -> None: ...
    async def send_node(self, node: Node) -> None: ...
    def set_force_active_delivery_receipts(self, active: bool) -> None: ...

# ── BlockingClient ──

class BlockingClient:
    async def block(self, jid: JID) -> None: ...
    async def unblock(self, jid: JID) -> None: ...
    async def get_blocklist(self) -> list[BlocklistEntry]: ...
    async def is_blocked(self, jid: JID) -> bool: ...

# ── ChatActionsClient ──

class ChatActionsClient:
    @staticmethod
    def build_message_key(id: str, remote_jid: JID, from_me: bool, participant: JID | None = None) -> _MessageKey: ...
    @staticmethod
    def build_message_range(last_message_timestamp: int, last_system_message_timestamp: int | None, messages: list[tuple[_MessageKey, int]]) -> _SyncActionMessageRange: ...
    async def archive_chat(self, jid: JID, message_range: Any | None = None) -> None: ...
    async def unarchive_chat(self, jid: JID, message_range: Any | None = None) -> None: ...
    async def pin_chat(self, jid: JID) -> None: ...
    async def unpin_chat(self, jid: JID) -> None: ...
    async def mute_chat(self, jid: JID) -> None: ...
    async def mute_chat_until(self, jid: JID, mute_end_timestamp_ms: int) -> None: ...
    async def unmute_chat(self, jid: JID) -> None: ...
    async def star_message(self, chat_jid: JID, participant_jid: JID | None, message_id: str, from_me: bool) -> None: ...
    async def unstar_message(self, chat_jid: JID, participant_jid: JID | None, message_id: str, from_me: bool) -> None: ...
    async def mark_chat_as_read(self, jid: JID, read: bool, message_range: Any | None = None) -> None: ...
    async def delete_chat(self, jid: JID, delete_media: bool, message_range: Any | None = None) -> None: ...
    async def delete_message_for_me(self, chat_jid: JID, participant_jid: JID | None, message_id: str, from_me: bool, delete_media: bool, message_timestamp: int | None) -> None: ...
    async def clear_chat(self, jid: JID, delete_starred: bool, delete_media: bool, message_range: Any | None = None) -> None: ...
    async def save_contact(self, jid: JID, full_name: str | None = None, first_name: str | None = None, save_on_primary_addressbook: bool = False) -> None: ...
    async def edit_message(self, chat_jid: JID, original_id: str, new_message: _Message) -> str: ...
    async def revoke_message(self, chat_jid: JID, message_id: str, original_sender: JID | None = None) -> None: ...
    async def react_message(self, chat_jid: JID, message_id: str, reaction: str, from_me: bool = False, participant_jid: JID | None = None) -> str: ...

# ── ChatstateClient ──

class ChatstateClient:
    async def send(self, to: JID, state: ChatStateType) -> None: ...
    async def send_composing(self, to: JID) -> None: ...
    async def send_recording(self, to: JID) -> None: ...
    async def send_paused(self, to: JID) -> None: ...

# ── CommentsClient ──

class CommentsClient:
    async def send_text(self, parent: EvMessage, text: str) -> str: ...
    async def send_message(self, parent: EvMessage, message: _Message) -> str: ...

# ── CommunityClient ──

class CommunityClient:
    @staticmethod
    def classify_group(metadata: GroupMetadata) -> GroupType: ...
    async def create(self, options: CreateCommunityOptions) -> CreateCommunityResult: ...
    async def deactivate(self, community_jid: JID) -> None: ...
    async def link_subgroups(self, community_jid: JID, subgroup_jids: list[JID]) -> LinkSubgroupsResult: ...
    async def unlink_subgroups(self, community_jid: JID, subgroup_jids: list[JID], remove_orphan_members: bool) -> UnlinkSubgroupsResult: ...
    async def get_subgroups(self, community_jid: JID) -> list[CommunitySubgroup]: ...
    async def get_subgroup_participant_counts(self, community_jid: JID) -> list[tuple[JID, int]]: ...
    async def query_linked_group(self, community_jid: JID, subgroup_jid: JID) -> GroupMetadata: ...
    async def join_subgroup(self, community_jid: JID, subgroup_jid: JID) -> GroupMetadata: ...
    async def get_linked_groups_participants(self, community_jid: JID) -> list[GroupParticipant]: ...

# ── ContactClient ──

class ContactClient:
    async def get_info(self, phones: list[str]) -> list[ContactInfo]: ...
    async def get_user_info(self, jid: JID) -> dict[JID, UserInfo]: ...
    async def get_profile_picture(self, jid: JID, preview: bool) -> ProfilePicture: ...
    async def is_on_whatsapp(self, jid: list[JID]) -> list[IsOnWhatsAppResult]: ...

# ── EventsClient ──

class EventsClient:
    async def create(self, chat_jid: JID, name: str, start_time: int | None = None, end_time: int | None = None, description: str | None = None, join_link: str | None = None, is_scheduled_call: bool | None = None, extra_guests_allowed: bool | None = None) -> dict[str, Any]: ...
    async def respond(self, chat_jid: JID, event_message_id: str, event_creator_jid: JID, message_secret: bytes, response: EventResponse, extra_guest_count: int | None = None) -> str: ...

class EventResponse:
    Going: ClassVar[EventResponse]
    NotGoing: ClassVar[EventResponse]
    Maybe: ClassVar[EventResponse]

# ── GroupsClient ──

class GroupsClient:
    async def query_info(self, jid: JID) -> GroupInfo: ...
    async def get_participating(self) -> dict[JID, GroupMetadata]: ...
    async def get_metadata(self, jid: JID) -> GroupMetadata: ...
    async def create_group(self, options: CreateGroupOptions) -> CreateGroupResult: ...
    async def set_subject(self, jid: JID, subject: str) -> None: ...
    async def set_description(self, jid: JID, description: str | None = None, prev: str | None = None) -> None: ...
    async def leave(self, jid: JID) -> None: ...
    async def add_participants(self, jid: JID, participants: list[JID]) -> list[ParticipantChangeResponse]: ...
    async def remove_participants(self, jid: JID, participants: list[JID]) -> list[ParticipantChangeResponse]: ...
    async def promote_participants(self, jid: JID, participants: list[JID]) -> None: ...
    async def demote_participants(self, jid: JID, participants: list[JID]) -> None: ...
    async def get_invite_link(self, jid: JID, reset: bool) -> str: ...
    async def set_locked(self, jid: JID, locked: bool) -> None: ...
    async def set_announce(self, jid: JID, announce: bool) -> None: ...
    async def set_ephemeral(self, jid: JID, expiration: int) -> None: ...
    async def set_membership_approval(self, jid: JID, mode: MembershipApprovalMode) -> None: ...
    async def join_with_invite_code(self, code: str) -> JoinGroupResult: ...
    async def join_with_invite_v4(self, group_jid: JID, code: str, expiration: int, admin_jid: JID) -> JoinGroupResult: ...
    async def get_invite_info(self, code: str) -> GroupMetadata: ...
    async def get_membership_requests(self, jid: JID) -> list[MembershipRequest]: ...
    async def approve_membership_requests(self, jid: JID, participants: list[JID]) -> list[ParticipantChangeResponse]: ...
    async def reject_membership_requests(self, jid: JID, participants: list[JID]) -> list[ParticipantChangeResponse]: ...
    async def set_member_add_mode(self, jid: JID, mode: MemberAddMode) -> None: ...
    async def set_no_frequently_forwarded(self, jid: JID, restrict: bool) -> None: ...
    async def set_allow_admin_reports(self, jid: JID, allow: bool) -> None: ...
    async def set_group_history(self, jid: JID, enabled: bool) -> None: ...
    async def set_member_link_mode(self, jid: JID, mode: MemberLinkMode) -> None: ...
    async def set_limit_sharing(self, jid: JID, enabled: bool) -> None: ...
    async def cancel_membership_requests(self, jid: JID, participants: list[JID]) -> list[ParticipantChangeResponse]: ...
    async def revoke_request_code(self, jid: JID, participants: list[JID]) -> list[ParticipantChangeResponse]: ...
    async def acknowledge(self, jid: JID) -> None: ...
    async def set_profile_picture(self, jid: JID, image_data: bytes) -> str: ...
    async def remove_profile_picture(self, jid: JID) -> str: ...
    async def update_member_label(self, jid: JID, label: str) -> None: ...

# ── LabelsClient ──

class LabelsClient:
    async def create_label(self, label_id: str, name: str, color: int) -> None: ...
    async def delete_label(self, label_id: str) -> None: ...
    async def add_chat_label(self, jid: JID, label_id: str) -> None: ...
    async def remove_chat_label(self, jid: JID, label_id: str) -> None: ...

# ── NewsletterClient ──

class NewsletterClient:
    async def get_admin_info(self, jid: JID) -> NewsletterAdminInfo: ...
    async def get_followers(self, jid: JID, count: int) -> list[NewsletterFollower]: ...
    async def list_subscribed(self) -> list[NewsletterMetadata]: ...
    async def get_metadata(self, jid: JID) -> NewsletterMetadata: ...
    async def get_metadata_by_invite(self, invite_code: str) -> NewsletterMetadata: ...
    async def create(self, name: str, description: str | None = None) -> NewsletterMetadata: ...
    async def join(self, jid: JID) -> NewsletterMetadata: ...
    async def leave(self, jid: JID) -> None: ...
    async def update(self, jid: JID, name: str | None = None, description: str | None = None) -> NewsletterMetadata: ...
    async def subscribe_live_updates(self, jid: JID) -> int: ...
    async def send_message(self, jid: JID, message: _Message) -> str: ...
    async def send_reaction(self, jid: JID, server_id: int, reaction: str) -> None: ...
    async def set_follower_mute(self, jid: JID, muted: bool) -> None: ...
    async def set_admin_mute(self, jid: JID, muted: bool) -> None: ...
    async def edit_message(self, jid: JID, message_id: str, message: _Message) -> None: ...
    async def revoke_message(self, jid: JID, message_id: str) -> None: ...
    async def get_messages(self, jid: JID, count: int, before: int | None = None) -> list[NewsletterMessage]: ...

# ── PollsClient ──

class PollsClient:
    @staticmethod
    def decrypt_vote(enc_payload: bytes, enc_iv: bytes, message_secret: bytes, poll_msg_id: str, poll_creator_jid: JID, voter_jid: JID) -> list[bytes]: ...
    @staticmethod
    def aggregate_votes(poll_options: list[str], votes: list[tuple[JID, bytes, bytes]], message_secret: bytes, poll_msg_id: str, poll_creator_jid: JID) -> list[PollOptionResult]: ...
    async def create(self, to: JID, name: str, options: list[str], selectable_count: int) -> tuple[str, bytes]: ...
    async def vote(self, chat_jid: JID, poll_msg_id: str, poll_creator_jid: JID, message_secret: bytes, option_names: list[str]) -> str: ...

# ── PresenceClient ──

class PresenceClient:
    async def set(self, status: PresenceStatus) -> None: ...
    async def set_available(self) -> None: ...
    async def set_unavailable(self) -> None: ...
    async def subscribe(self, jid: JID) -> None: ...
    async def unsubscribe(self, jid: JID) -> None: ...

# ── PrivacyClient ──

class PrivacyClient:
    async def fetch_settings(self) -> list[PrivacySetting]: ...
    async def set_setting(self, category: PrivacyCategory, value: PrivacyValue) -> None: ...
    async def set_disallowed_list(self, category: PrivacyCategory, update: DisallowedListUpdate) -> None: ...
    async def set_default_disappearing_mode(self, duration_seconds: int) -> None: ...

# ── ProfileClient ──

class ProfileClient:
    async def set_push_name(self, name: str) -> None: ...
    async def set_status_text(self, text: str) -> None: ...
    async def set_profile_picture(self, image_data: bytes) -> str: ...
    async def remove_profile_picture(self) -> str: ...

# ── StatusClient ──

class StatusClient:
    @staticmethod
    def default_privacy() -> StatusPrivacySetting: ...
    async def send_text(self, text: str, background_argb: int, font: int, recipients: list[JID], options: StatusSendOptions | None = None) -> str: ...
    async def send_image(self, upload: UploadResponse, thumbnail: bytes, recipients: list[JID], caption: str | None = None, options: StatusSendOptions | None = None) -> str: ...
    async def send_video(self, upload: UploadResponse, thumbnail: bytes, duration_seconds: int, recipients: list[JID], caption: str | None = None, options: StatusSendOptions | None = None) -> str: ...
    async def send_raw(self, message: Any, recipients: list[JID], options: StatusSendOptions | None = None) -> str: ...
    async def revoke(self, message_id: str, recipients: list[JID], options: StatusSendOptions | None = None) -> str: ...

# ── VoIP Types ──

class CallHandle:
    """Handle for an active VoIP call."""
    call_id: str
    peer: JID
    def is_muted(self) -> bool: ...
    def set_muted(self, muted: bool) -> None: ...
    async def hangup(self) -> None: ...
    async def wait_ended(self) -> None: ...
    async def start_video(self, video_source: Any, video_sink: Any) -> None: ...
    async def stop_video(self) -> None: ...
    async def invite_participant(self, target: JID) -> None: ...
    async def ring_participant(self, target: JID) -> None: ...
    async def start_screen_share(self, screen_share_id: int | None = None) -> None: ...
    async def stop_screen_share(self) -> None: ...
    async def set_approval_required(self, enabled: bool) -> None: ...
    async def admit_waiting_user(self, target: JID) -> None: ...
    async def deny_waiting_user(self, target: JID) -> None: ...

class IncomingCallEvent:
    """Incoming call event received by the client."""
    call_id: str
    peer: JID
    is_video: bool
    async def reject(self) -> None: ...
    async def accept(self, audio_source: Any, audio_sink: Any) -> CallHandle: ...

class VoipClient:
    """VoIP client for voice and video calls."""
    async def call(self, peer: JID, audio_source: Any, audio_sink: Any) -> CallHandle: ...
    async def video_call(self, peer: JID, audio_source: Any, audio_sink: Any, video_source: Any, video_sink: Any) -> CallHandle: ...
    async def group_call(self, peers: list[JID], audio_source: Any, audio_sink: Any, video_source: Any | None = None, video_sink: Any | None = None) -> CallHandle: ...
    async def join_call_link(self, token_or_url: str, media: str, audio_source: Any, audio_sink: Any, video_source: Any | None = None, video_sink: Any | None = None) -> CallHandle: ...

# ── Media Players ──

class AudioPlayer:
    """Audio player for playback of voice notes, media, etc."""
    state: str
    def __init__(self) -> None: ...
    async def play(self, path: str, mode: str = ...) -> None: ...
    def pause(self) -> None: ...
    def resume(self) -> None: ...
    def stop(self) -> None: ...
    def skip(self) -> None: ...
    def clear_queue(self) -> None: ...
    async def enqueue(self, path: str) -> None: ...

class VideoPlayer:
    """Video player for playback of video messages."""
    def __init__(self) -> None: ...
    async def play(self, path: str) -> None: ...
    def stop(self) -> None: ...
