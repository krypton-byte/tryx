"""Event classes and payload types emitted by the Tryx runtime."""

from datetime import datetime
from typing import Any, Awaitable, Callable, TypeVar

from .types import JID, MessageInfo, MessageSource
from .wacore import BusinessSubscription, KeyIndexInfo, Node
from .waproto.whatsapp_pb2 import Conversation, HistorySync, SyncActionValue
from .waproto.whatsapp_pb2 import Message as MessageProto

_ArchiveChatAction = SyncActionValue.ArchiveChatAction
_ContactAction = SyncActionValue.ContactAction
_DeleteChatAction = SyncActionValue.DeleteChatAction
_DeleteMessageForMeAction = SyncActionValue.DeleteMessageForMeAction
_MarkChatAsReadAction = SyncActionValue.MarkChatAsReadAction
_MuteAction = SyncActionValue.MuteAction

EventT = TypeVar("EventT")

class Dispatcher:
    """Callback registry used by the runtime to map event classes to handlers."""

    def __init__(self) -> None:
        """
        Create an empty dispatcher with no registered handlers.
        """
        ...
    def on(self, event_type: type[EventT]) -> Dispatcher:
        """Select an event class and return a decorator-like dispatcher object."""
        ...

    def __call__(
        self,
        func: Callable[..., Awaitable[None]] | Callable[..., Any],
    ) -> Callable[..., Awaitable[None]] | Callable[..., Any]:
        """Register a callback function for the previously selected event class.

        The callback receives ``(client, event)`` as positional arguments.
        """
        ...

class TempBanReason:
    """Reason category for temporary account restrictions."""

    SentToTooManyPeople: TempBanReason
    SentBlockedNyUser: TempBanReason
    CreateTooManyGroups: TempBanReason
    SentTooManySameMessage: TempBanReason
    Unknown: TempBanReason

class ReceiptType:
    """Receipt status type for incoming receipt events."""

    Delivered: ReceiptType
    Sender: ReceiptType
    Retry: ReceiptType
    Read: ReceiptType
    ReadSelf: ReceiptType
    Played: ReceiptType
    PlayedSelf: ReceiptType
    ServerError: ReceiptType
    Inactive: ReceiptType
    PeerMsg: ReceiptType
    HistorySync: ReceiptType
    EncRekeyRetry: ReceiptType
    Other: ReceiptType

class UnavailableType:
    """Unavailable media category for undecryptable message events."""

    Unknown: UnavailableType
    ViewOnce: UnavailableType

class DecryptFailMode:
    """Client behavior hint when decryption fails."""

    Show: DecryptFailMode
    Hide: DecryptFailMode

class ChatPresence:
    """Presence activity state for a chat."""

    Composing: ChatPresence
    Paused: ChatPresence

class ChatPresenceMedia:
    """Media kind associated with chat presence activity."""

    Text: ChatPresenceMedia
    Audio: ChatPresenceMedia

class DeviceListUpdateType:
    """Type of device list change in multi-device sync events."""

    Added: DeviceListUpdateType
    Removed: DeviceListUpdateType
    Updated: DeviceListUpdateType

class BusinessStatusUpdateType:
    """Business profile update categories."""

    RemovedAsBusiness: BusinessStatusUpdateType
    VerifiedNameChanged: BusinessStatusUpdateType
    ProfileUpdated: BusinessStatusUpdateType
    ProductsUpdated: BusinessStatusUpdateType
    CollectionsUpdated: BusinessStatusUpdateType
    SubscriptionsUpdated: BusinessStatusUpdateType
    Unknown: BusinessStatusUpdateType

class GroupNotificationAction:
    """Opaque group update action variant.

    The runtime returns one action object that describes what changed in a group.
    """

class EvConnected:
    """Emitted when the session becomes connected."""

class EvDisconnected:
    """Emitted when the session disconnects."""

class EvLoggedOut:
    """Emitted when the account logs out."""

    on_connect: bool
    reason: str

class PairSuccessData:
    """Pairing success payload."""

    id: JID
    lid: JID
    business_name: str
    platform: str

class EvPairSuccess:
    """Emitted when account pairing succeeds."""

    @property
    def data(self) -> PairSuccessData:
        """
        Return the pairing success payload.

        Returns:
            PairSuccessData with id, lid, business_name, and platform.
        """
        ...

class EvPairError:
    """Emitted when account pairing fails."""

    id: JID
    lid: JID
    business_name: str
    platform: str
    error: str

class EvPairingQrCode:
    """Contains QR pairing code payload."""

    code: str
    timeout: int

class EvPairingCode:
    """Contains numeric pairing code payload."""

    code: str
    timeout: int

class EvQrScannedWithoutMultidevice:
    """Emitted when a QR code is scanned without multidevice support."""

class EvClientOutDated:
    """Emitted when the client version is considered outdated."""

class EvStreamReplaced:
    """Emitted when the active stream/session is replaced by another login."""

class EvTemporaryData:
    """Payload for temporary ban details."""

    code: TempBanReason
    expire: datetime

class EvTemporaryBan:
    """Emitted when the account receives a temporary ban."""

    @property
    def data(self) -> EvTemporaryData:
        """
        Return the temporary ban details.

        Returns:
            EvTemporaryData with ban reason code and expiration time.
        """
        ...

class EvConnectFailure:
    """Emitted when an initial connect attempt fails."""

    reason: str
    message: str | None

    @property
    def node(self) -> Node | None:
        """
        Return the raw protocol node, or ``None`` if unavailable.

        Returns:
            Optional protocol Node for debugging connection failures.
        """
        ...

class EvStreamError:
    """Emitted when stream-level protocol error is received."""

    code: str

    @property
    def node(self) -> Node | None:
        """
        Return the raw protocol node, or ``None`` if unavailable.

        Returns:
            Optional protocol Node for debugging stream errors.
        """
        ...

class EvReceipt:
    """Message receipt update event."""

    message_ids: list[str]
    timestamp: datetime
    receipt_type: ReceiptType
    message_sender: JID

    @property
    def source(self) -> MessageSource | None:
        """
        Return the message source, or ``None`` if unavailable.

        Returns:
            Optional MessageSource with sender/chat JIDs.
        """
        ...

class EvUndecryptableMessage:
    """Emitted when a message cannot be decrypted."""

    is_unavailable: bool
    unavailable_type: UnavailableType
    decrypt_fail_mode: DecryptFailMode

    @property
    def info(self) -> MessageInfo | None:
        """
        Return message metadata, or ``None`` if unavailable.

        Returns:
            Optional MessageInfo for the undecryptable message.
        """
        ...

class MessageData:
    """Normalized message payload data."""

    conversation: str | None
    caption: str | None

    @property
    def message_info(self) -> MessageInfo:
        """
        Return normalized message metadata (id, type, push_name, source).

        Returns:
            MessageInfo for this message.
        """
        ...
    def get_extended_text_message(self) -> str | None:
        """
        Extract text from extended text message, or ``None``.

        Returns:
            The extended text content, or ``None``.
        """
        ...
    def get_text(self) -> str | None:
        """
        Extract plain text from the message body.

        Returns:
            The message text, or ``None`` if not a text message.
        """
        ...
    @property
    def raw_proto(self) -> MessageProto:
        """
        Return the raw protobuf message object.

        Returns:
            The underlying protobuf Message instance.
        """
        ...

class EvMessage:
    """Main message event."""

    @property
    def data(self) -> MessageData:
        """
        Return the normalized message payload.

        Returns:
            MessageData with text, caption, and metadata.
        """
        ...

class EvNotification:
    """Raw notification node event."""

    @property
    def node(self) -> Node:
        """
        Return the raw notification protocol node.

        Returns:
            The notification Node for custom handling.
        """
        ...

class EvChatPresence:
    """Typing/recording presence event for a chat."""

    @property
    def source(self) -> MessageSource:
        """
        Return the chat source (sender and chat JIDs).

        Returns:
            MessageSource identifying the chat.
        """
        ...
    @property
    def state(self) -> str:
        """
        Return the presence state string (e.g. ``'composing'``).

        Returns:
            State string like ``'composing'``, ``'paused'``.
        """
        ...
    @property
    def media(self) -> str:
        """
        Return the presence media type (e.g. ``'text'``).

        Returns:
            Media string like ``'text'``, ``'audio'``.
        """
        ...

class EvPresence:
    """Presence update event for a contact."""

    from_: JID
    unavailable: bool
    last_seen: datetime | None

class PictureUpdateData:
    """Profile picture update payload."""

    jid: JID
    author: JID | None
    removed: bool
    timestamp: datetime | None
    picture_id: str | None

class EvPictureUpdate:
    """Emitted when profile picture changes."""

    @property
    def data(self) -> PictureUpdateData:
        """
        Return the picture update payload.

        Returns:
            PictureUpdateData with jid, author, and picture_id.
        """
        ...

class UserAboutUpdateData:
    """User bio/about text update payload."""

    jid: JID
    status: str
    timestamp: datetime | None

class EvUserAboutUpdate:
    """Emitted when a user's about/status text changes."""

    @property
    def data(self) -> UserAboutUpdateData:
        """
        Return the about/status text update payload.

        Returns:
            UserAboutUpdateData with jid and status text.
        """
        ...

class LazyConversation:
    """Deferred conversation object from history sync."""

    @property
    def conversation(self) -> Conversation | None:
        """
        Return the parsed Conversation protobuf, or ``None``.

        Returns:
            Optional Conversation proto from history sync.
        """
        ...

class EvJoinedGroup:
    """Emitted when account joins a group."""

    @property
    def data(self) -> LazyConversation:
        """
        Return the lazy conversation payload.

        Returns:
            LazyConversation for the joined group event.
        """
        ...

class EvGroupInfoUpdate:
    """Emitted for generic group info changes."""

class EvPushNameUpdateData:
    """Push name change payload for contact updates."""

    jid: JID
    message: MessageInfo
    old_push_name: str
    new_push_name: str

class EvPushNameUpdate:
    """Emitted when a contact push name changes."""

    @property
    def data(self) -> EvPushNameUpdateData:
        """
        Return the push name change payload.

        Returns:
            EvPushNameUpdateData with old and new push names.
        """
        ...

class EvSelfPushNameUpdated:
    """Emitted when own account push name is updated."""

    from_server: bool
    old_name: str
    new_name: str

class PinUpdatedata:
    """Pin update payload."""

    jid: JID
    timestamp: datetime
    pinned: bool | None
    from_full_sync: bool

class EvPinUpdate:
    """Emitted when chat pin status changes."""

    @property
    def data(self) -> PinUpdatedata:
        """
        Return the pin update payload.

        Returns:
            PinUpdatedata with jid, timestamp, and pinned state.
        """
        ...

class MuteUpdateData:
    """Mute update payload."""

    jid: JID
    timestamp: datetime
    from_full_sync: bool

    @property
    def action(self) -> _MuteAction:
        """
        Return the mute action protobuf.

        Returns:
            SyncActionValue.MuteAction proto.
        """
        ...

class EvMuteUpdate:
    """Emitted when mute settings change."""

    @property
    def data(self) -> MuteUpdateData:
        """
        Return the mute update payload.

        Returns:
            MuteUpdateData with jid and timestamp.
        """
        ...

class MarkChatAsReadUpdateData:
    """Read/unread marker sync payload."""

    jid: JID
    timestamp: datetime
    from_full_sync: bool

    @property
    def action(self) -> _MarkChatAsReadAction:
        """
        Return the mark-as-read action protobuf.

        Returns:
            SyncActionValue.MarkChatAsReadAction proto.
        """
        ...

class EvMarkChatAsReadUpdate:
    """Emitted when read state sync action is applied."""

    @property
    def data(self) -> MarkChatAsReadUpdateData:
        """
        Return the mark-as-read update payload.

        Returns:
            MarkChatAsReadUpdateData with jid and timestamp.
        """
        ...

class EvHistorySync:
    """Contains protobuf history sync payload."""

    @property
    def proto(self) -> HistorySync:
        """
        Return the raw HistorySync protobuf.

        Returns:
            HistorySync proto containing synced messages.
        """
        ...

class OfflineSyncData:
    """Preview counters for offline sync."""

    total: int
    app_data_changes: int
    messages: int
    notifications: int
    receipts: int

class EvOfflineSyncPreview:
    """Emitted before offline sync processing starts."""

    @property
    def data(self) -> OfflineSyncData:
        """
        Return offline sync preview counters.

        Returns:
            OfflineSyncData with total, messages, notifications counts.
        """
        ...

class OfflineSyncCompletedData:
    """Summary payload after offline sync completes."""

    count: int

class EvOfflineSyncCompleted:
    """Emitted when offline sync is fully processed."""

    @property
    def data(self) -> OfflineSyncCompletedData:
        """
        Return offline sync completion summary.

        Returns:
            OfflineSyncCompletedData with processed count.
        """
        ...

class DeviceNottificationInfo:
    """Single device info entry within a device list update."""

    device_id: int
    key_index: int | None

class DeviceListUpdateData:
    """Device list synchronization payload."""

    user: JID
    lid_user: JID | None
    update_type: DeviceListUpdateType
    devices: list[DeviceNottificationInfo]
    key_index: KeyIndexInfo | None
    contact_hash: str | None

class EvDeviceListUpdate:
    """Emitted when companion device list changes."""

    @property
    def data(self) -> DeviceListUpdateData:
        """
        Return the device list update payload.

        Returns:
            DeviceListUpdateData with user, devices, and update type.
        """
        ...

class BusinessStatusUpdateData:
    """Business profile sync payload."""

    jid: JID
    update_type: BusinessStatusUpdateType
    timestamp: datetime
    target_jid: JID | None
    hash: str | None
    product_ids: list[str]
    collection_ids: list[str]
    subscriptions: list[BusinessSubscription]

class EvBusinessStatusUpdate:
    """Emitted when business profile information changes."""

    @property
    def data(self) -> BusinessStatusUpdateData:
        """
        Return the business profile update payload.

        Returns:
            BusinessStatusUpdateData with jid and update type.
        """
        ...

class EvArchiveUpdateData:
    """Archive state sync payload."""

    jid: JID
    timestamp: datetime
    from_full_sync: bool

    @property
    def action(self) -> _ArchiveChatAction:
        """
        Return the archive action protobuf.

        Returns:
            SyncActionValue.ArchiveChatAction proto.
        """
        ...

class EvArchiveUpdate:
    """Emitted when chat archive state changes."""

    @property
    def data(self) -> EvArchiveUpdateData:
        """
        Return the archive update payload.

        Returns:
            EvArchiveUpdateData with jid and timestamp.
        """
        ...

class EvDisappearingModeChangedData:
    """Disappearing mode update payload."""

    from_: JID
    duration: int
    setting_timestamp: int

class EvDisappearingModeChanged:
    """Emitted when disappearing mode duration changes."""

    @property
    def data(self) -> EvDisappearingModeChangedData:
        """
        Return the disappearing mode change payload.

        Returns:
            EvDisappearingModeChangedData with duration and timestamp.
        """
        ...

class EvContactNumberChangedData:
    """Contact number change payload."""

    old_jid: JID
    new_jid: JID
    old_lid: JID | None
    new_lid: JID | None
    timestamp: datetime

class EvContactNumberChanged:
    """Emitted when a contact number/JID is migrated."""

    @property
    def data(self) -> EvContactNumberChangedData:
        """
        Return the contact number change payload.

        Returns:
            EvContactNumberChangedData with old and new JIDs.
        """
        ...

class EvContactSyncRequestedData:
    """Payload that indicates contact sync was requested."""

    after: datetime | None
    timestamp: datetime

class EvContactSyncRequested:
    """Emitted when the server requests contact synchronization."""

    @property
    def data(self) -> EvContactSyncRequestedData:
        """
        Return the contact sync request payload.

        Returns:
            EvContactSyncRequestedData with after timestamp.
        """
        ...

class EvContactUpdatedData:
    """Payload for single contact metadata updates."""

    jid: JID
    timestamp: datetime

class EvContactUpdated:
    """Emitted when a contact metadata entry is updated."""

    @property
    def data(self) -> EvContactUpdatedData:
        """
        Return the contact update payload.

        Returns:
            EvContactUpdatedData with jid and timestamp.
        """
        ...

class EvStarUpdateData:
    """Star/unstar sync payload for a specific message."""

    chat_jid: JID
    participant_jid: JID | None
    message_id: str
    from_me: bool
    timestamp: datetime
    from_full_sync: bool
    starred: bool | None

class EvStarUpdate:
    """Emitted when message star state changes."""

    @property
    def data(self) -> EvStarUpdateData:
        """
        Return the star update payload.

        Returns:
            EvStarUpdateData with message_id, starred state, and timestamp.
        """
        ...

class GroupParticipant:
    """Participant entry embedded in group notification actions."""

    jid: JID
    phone_number: JID | None

class GroupUpdateData:
    """Group metadata/action update payload."""

    group_jid: JID
    participant: JID | None
    participant_pn: JID | None
    timestamp: datetime
    is_lid_addressing_mode: bool
    action: GroupNotificationAction

class EvGroupUpdate:
    """Emitted for rich group notification changes."""

    @property
    def data(self) -> GroupUpdateData:
        """
        Return the group notification update payload.

        Returns:
            GroupUpdateData with group_jid, participant, and action.
        """
        ...

class ContactUpdateData:
    """Contact sync action payload."""

    jid: JID
    timestamp: datetime
    from_full_sync: bool

    @property
    def action(self) -> _ContactAction:
        """
        Return the contact sync action protobuf.

        Returns:
            SyncActionValue.ContactAction proto.
        """
        ...

class EvContactUpdate:
    """Emitted when contact sync actions are applied."""

    @property
    def data(self) -> ContactUpdateData:
        """
        Return the contact update payload.

        Returns:
            ContactUpdateData with jid and timestamp.
        """
        ...

class NewsletterLiveUpdateReaction:
    """Reaction count entry in newsletter live updates."""

    code: str
    count: int

class NewsletterUpdateMessage:
    """Newsletter message snapshot used by live update events."""

    server_id: int
    reactions: list[NewsletterLiveUpdateReaction]

class NewsletterLiveUpdateData:
    """Payload for newsletter live update events."""

    newsletter_jid: JID
    messages: list[NewsletterUpdateMessage]

class EvNewsletterLiveUpdate:
    """Emitted when subscribed newsletter receives live changes."""

    @property
    def data(self) -> NewsletterLiveUpdateData:
        """
        Return the newsletter live update payload.

        Returns:
            NewsletterLiveUpdateData with messages and reactions.
        """
        ...

class DeleteChatUpdateData:
    """Delete-chat sync action payload."""

    jid: JID
    delete_media: bool
    timestamp: datetime
    from_full_sync: bool

    @property
    def action(self) -> _DeleteChatAction:
        """
        Return the delete-chat action protobuf.

        Returns:
            SyncActionValue.DeleteChatAction proto.
        """
        ...

class EvDeleteChatUpdate:
    """Emitted when a chat is deleted via sync action."""

    @property
    def data(self) -> DeleteChatUpdateData:
        """
        Return the delete-chat update payload.

        Returns:
            DeleteChatUpdateData with jid and timestamp.
        """
        ...

class DeleteMessageForMeUpdateData:
    """Delete-for-me sync action payload for a single message."""

    chat_jid: JID
    participant_jid: JID | None
    message_id: str
    from_me: bool
    timestamp: datetime
    action: _DeleteMessageForMeAction
    from_full_sync: bool

class EvDeleteMessageForMeUpdate:
    """Emitted when a message is deleted-for-me via sync action."""

    @property
    def data(self) -> DeleteMessageForMeUpdateData:
        """
        Return the delete-for-me update payload.

        Returns:
            DeleteMessageForMeUpdateData with message_id and timestamp.
        """
        ...
