from datetime import datetime
from typing import Any
from .types import JID, MessageInfo, MessageSource
from .waproto.whatsapp_pb2 import (
    Conversation,
    HistorySync,
    Message as MessageProto,
    SyncActionValue,
)

class EvConnected: ...
class EvDisconnected: ...

class EvLoggedOut:
    on_connect: bool
    reason: str

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

class EvPairingQrCode:
    code: str
    timeout: int

class EvPairingCode:
    code: str
    timeout: int

class EvQrScannedWithoutMultidevice: ...
class EvClientOutDated: ...
class EvStreamReplaced: ...

class EvTemporaryData:
    code: object
    expire: datetime

class EvTemporaryBan:
    @property
    def data(self) -> EvTemporaryData: ...

class EvConnectFailure:
    reason: str
    message: str
    @property
    def node(self) -> Any | None: ...

class EvStreamError:
    code: str
    @property
    def node(self) -> Any | None: ...

class EvReceipt:
    message_ids: list[str]
    timestamp: datetime
    receipt_type: object
    message_sender: JID
    @property
    def source(self) -> MessageSource | None: ...

class EvUndecryptableMessage:
    is_unavailable: bool
    unavailable_type: object
    decrypt_fail_mode: object
    @property
    def info(self) -> MessageInfo | None: ...

class MessageData:
    conversation: str | None
    caption: str | None
    @property
    def message_info(self) -> MessageInfo: ...
    def get_extended_text_message(self) -> str | None: ...
    def get_text(self) -> str | None: ...
    @property
    def raw_proto(self) -> MessageProto: ...

class EvMessage:
    @property
    def data(self) -> MessageData: ...

class EvNotification:
    @property
    def node(self) -> Any: ...

class EvChatPresence:
    @property
    def source(self) -> MessageSource: ...
    @property
    def state(self) -> str: ...
    @property
    def media(self) -> str: ...

class EvPresence:
    from_: JID
    unavailable: bool
    last_seen: datetime | None

class PictureUpdateData:
    jid: JID
    author: JID | None
    removed: bool
    timestamp: datetime | None
    picture_id: str | None

class EvPictureUpdate:
    @property
    def data(self) -> PictureUpdateData: ...

class UserAboutUpdateData:
    jid: JID
    status: str
    timestamp: datetime | None

class EvUserAboutUpdate:
    @property
    def data(self) -> UserAboutUpdateData: ...

class LazyConversation:
    @property
    def conversation(self) -> Conversation | None: ...

class EvJoinedGroup:
    @property
    def data(self) -> LazyConversation: ...

class EvGroupInfoUpdate: ...

class EvPushNameUpdateData:
    jid: JID
    message: MessageInfo
    old_push_name: str
    new_push_name: str

class EvPushNameUpdate:
    @property
    def data(self) -> EvPushNameUpdateData: ...

class EvSelfPushNameUpdated:
    from_server: bool
    old_name: str
    new_name: str

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
    def action(self) -> Any: ...

class EvMuteUpdate:
    @property
    def data(self) -> MuteUpdateData: ...

class MarkChatAsReadUpdateData:
    jid: JID
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> Any: ...

class EvMarkChatAsReadUpdate:
    @property
    def data(self) -> MarkChatAsReadUpdateData: ...

class EvHistorySync:
    @property
    def proto(self) -> HistorySync: ...

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

class DeviceListUpdateData:
    user: JID
    lid_user: JID | None
    update_type: object
    devices: list[object]
    key_index: object | None
    contact_hash: str | None

class EvDeviceListUpdate:
    @property
    def data(self) -> DeviceListUpdateData: ...

class BusinessStatusUpdateData:
    jid: JID
    update_type: object
    timestamp: datetime
    target_jid: JID | None
    hash: str | None
    product_ids: list[str]
    collection_ids: list[str]
    subscriptions: list[object]

class EvBusinessStatusUpdate:
    @property
    def data(self) -> BusinessStatusUpdateData: ...

class EvArchiveUpdateData:
    jid: JID
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> Any: ...

class EvArchiveUpdate:
    @property
    def data(self) -> EvArchiveUpdateData: ...

class EvDisappearingModeChangedData:
    from_: JID
    duration: int
    setting_timestamp: int

class EvDisappearingModeChanged:
    @property
    def data(self) -> EvDisappearingModeChangedData: ...

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

class EvContactUpdatedData:
    jid: JID
    timestamp: datetime

class EvContactUpdated:
    @property
    def data(self) -> EvContactUpdatedData: ...

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

class GroupUpdateData:
    group_jid: JID
    participant: JID | None
    participant_pn: JID | None
    timestamp: datetime
    is_lid_addressing_mode: bool
    action: object

class EvGroupUpdate:
    @property
    def data(self) -> GroupUpdateData: ...

class EvContactUpdate:
    @property
    def data(self) -> Any: ...

class EvNewsletterLiveUpdateData:
    newsletter_jid: JID
    messages: list[Any]

class EvNewsletterLiveUpdate:
    @property
    def data(self) -> EvNewsletterLiveUpdateData: ...

class EvDeleteChatUpdateData:
    jid: JID
    delete_media: bool
    timestamp: datetime
    from_full_sync: bool
    @property
    def action(self) -> Any: ...

class EvDeleteChatUpdate:
    @property
    def data(self) -> EvDeleteChatUpdateData: ...

class DeleteMessageForMeUpdateData:
    chat_jid: JID
    participant_jid: JID | None
    message_id: str
    from_me: bool
    timestamp: datetime
    action: Any
    from_full_sync: bool

class EvDeleteMessageForMeUpdate:
    @property
    def data(self) -> DeleteMessageForMeUpdateData: ...
