from typing import Any, Awaitable, Callable, Type, TypeVar

from .backend import BackendBase
from .events import EvMessage
from .types import JID, ProfilePicture, UploadResponse
from .wacore import MediaType
from .waproto.whatsapp_pb2 import Message as MessageProto

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
    def is_connected(self) -> bool: ...
    async def get_user_info(self, jid: JID) -> dict[JID, UserInfo]: ...
    async def get_profile_picture(self, jid: JID, preview: bool) -> ProfilePicture: ...
    async def is_on_whatsapp(self, jid: list[JID]) -> list[IsOnWhatsAppResult]: ...
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
    
