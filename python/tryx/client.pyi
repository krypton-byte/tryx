# from ast import TypeVar
from types import CoroutineType
from typing import Any, Awaitable, Callable, Type

# from tryx.events import Message
from .waproto.whatsapp_pb2 import Message as MessageProto
from .types import JID, UploadResponse
from .backend import SqliteBackend
from .wacore import MediaType

class Tryx:
    def __init__(self, backend: SqliteBackend) -> None: ...
    def on[T](
        self, event_type: Type[T]
    ) -> Callable[
        [Callable[[TryxClient, T], CoroutineType[None, None, Any | None]]],
        Callable[[TryxClient, T], CoroutineType[None, None, Any | None]],
    ]: ...
    def run(self) -> Awaitable[None]: ...
    def run_blocking(self) -> None: ...

DownloadableMedia = (
    MessageProto.ImageMessage
    | MessageProto.VideoMessage
    | MessageProto.AudioMessage
    | MessageProto.DocumentMessage
    | MessageProto.StickerMessage
)

class TryxClient:
    async def send_message(self, chat: JID, message: MessageProto) -> str: ...
    async def upload(self, data: bytes, media_type: MediaType) -> UploadResponse: ...
    async def upload_file(self, path: str, media_type: MediaType) -> UploadResponse: ...
    async def download_media(self, message: DownloadableMedia) -> bytes: ...
    async def send_image(
        self,
        to: JID,
        photo_data: bytes,
        caption: str,
        quoted: MessageProto | None = None,
    ) -> str: ...
    async def send_text(
        self,
        to: JID,
        text: str,
        quoted: MessageProto | None = None,
    ) -> str: ...
    
