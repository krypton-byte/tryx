from typing import Any, Awaitable, Callable, Type, TypeVar

from .backend import BackendBase
from .events import EvMessage
from .types import JID, ProfilePicture, UploadResponse
from .wacore import MediaType
from .waproto.whatsapp_pb2 import Message as MessageProto
from .waproto.whatsapp_pb2 import MessageKey, SyncActionMessageRange

EventT = TypeVar("EventT")


class IsOnWhatsAppResult:
    jid: JID
    is_registered: bool


class UserInfo:
    jid: JID
    lid: JID | None
    status: str | None
    picture_id: str | None
    is_business: bool


class ContactInfo:
    jid: JID
    lid: JID | None
    is_registered: bool
    is_business: bool
    status: str | None
    picture_id: int | None


DownloadableMedia = (
    MessageProto.ImageMessage
    | MessageProto.VideoMessage
    | MessageProto.AudioMessage
    | MessageProto.DocumentMessage
    | MessageProto.StickerMessage
)


class Tryx:
    handlers: Any

    def __init__(self, backend: BackendBase) -> None: ...
    def get_client(self) -> TryxClient: ...
    def on(
        self, event_type: Type[EventT]
    ) -> Callable[[Callable[..., Awaitable[Any]]], Callable[..., Awaitable[Any]]]: ...
    def run(self) -> Awaitable[None]: ...
    def run_blocking(self) -> None: ...


class TryxClient:
    contact: ContactClient
    chat_actions: ChatActionsClient

    def is_connected(self) -> bool: ...
    async def download_media(self, message: DownloadableMedia) -> bytes: ...
    async def upload_file(self, path: str, media_type: MediaType) -> UploadResponse: ...
    async def upload(self, data: bytes, media_type: MediaType) -> UploadResponse: ...
    async def send_message(self, to: JID, message: MessageProto) -> str: ...
    async def send_text(self, to: JID, text: str, quoted: EvMessage | None = None) -> str: ...
    async def send_photo(
        self,
        to: JID,
        photo_data: bytes,
        caption: str,
        quoted: EvMessage | None = None,
    ) -> str: ...


class ContactClient:
    async def get_info(self, phones: list[str]) -> list[ContactInfo]: ...
    async def get_user_info(self, jid: JID) -> dict[JID, UserInfo]: ...
    async def get_profile_picture(self, jid: JID, preview: bool) -> ProfilePicture: ...
    async def is_on_whatsapp(self, jid: list[JID]) -> list[IsOnWhatsAppResult]: ...


class ChatActionsClient:
    @staticmethod
    def build_message_key(
        id: str,
        remote_jid: JID,
        from_me: bool,
        participant: JID | None = None,
    ) -> MessageKey: ...
    @staticmethod
    def build_message_range(
        last_message_timestamp: int,
        last_system_message_timestamp: int | None,
        messages: list[tuple[MessageKey, int]],
    ) -> SyncActionMessageRange: ...
    async def archive_chat(
        self,
        jid: JID,
        message_range: SyncActionMessageRange | None = None,
    ) -> None: ...
    async def unarchive_chat(
        self,
        jid: JID,
        message_range: SyncActionMessageRange | None = None,
    ) -> None: ...
    async def pin_chat(self, jid: JID) -> None: ...
    async def unpin_chat(self, jid: JID) -> None: ...
    async def mute_chat(self, jid: JID) -> None: ...
    async def mute_chat_until(self, jid: JID, mute_end_timestamp_ms: int) -> None: ...
    async def unmute_chat(self, jid: JID) -> None: ...
    async def star_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None: ...
    async def unstar_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None: ...
    async def mark_chat_as_read(
        self,
        jid: JID,
        read: bool,
        message_range: SyncActionMessageRange | None = None,
    ) -> None: ...
    async def delete_chat(
        self,
        jid: JID,
        delete_media: bool,
        message_range: SyncActionMessageRange | None = None,
    ) -> None: ...
    async def delete_message_for_me(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
        delete_media: bool,
        message_timestamp: int | None = None,
    ) -> None: ...

