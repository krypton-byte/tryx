# from ast import TypeVar
from types import CoroutineType
from typing import Any, Awaitable, Callable, Type

# from tryx.events import Message
from .waproto.whatsapp_pb2 import Message  as MessageProto
from .types import JID, UploadResponse
from .backend import SqliteBackend
from .wacore import MediaType

class Tryx:
    def __init__(self, backend: SqliteBackend) -> None: ...

    def on[T](self, event_type: Type[T]) -> Callable[[Callable[[TryxClient, T], CoroutineType[None, None, Any| None]]], Callable[[TryxClient, T], CoroutineType[None, None, Any | None]]]: ...

    def run(self) -> Awaitable[None]: ...
    def run_blocking(self) -> None: ...

class TryxClient:
    async def send_message(self, chat: JID, message: MessageProto) -> str: ...
    async def upload(self, data: bytes, media_type: MediaType) -> UploadResponse: ...
    async def upload_file(self, path: str, media_type: MediaType) -> UploadResponse: ...


class Nu[T]:
    X: T
    pass

K = Nu[int]()
class Test:
    def on[L: int](self, event_type: Nu[L]) -> Callable[[Callable[[TryxClient, L], CoroutineType[None, None, Any| None]]], Callable[[TryxClient, L], CoroutineType[None, None, Any | None]]]: ...

