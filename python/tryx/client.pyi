# from ast import TypeVar
from types import CoroutineType
from typing import Any, Awaitable, Callable, Type

# from tryx.events import Message
from .waproto.whatsapp_pb2 import Message  as MessageProto
from .types import JID
from .backend import SqliteBackend

class Tryx:
    def __init__(self, backend: SqliteBackend) -> None: ...

    def on[T](self, event_type: Type[T]) -> Callable[[Callable[[TryxClient, T], CoroutineType[None, None, Any| None]]], Callable[[TryxClient, T], CoroutineType[None, None, Any | None]]]: ...

    def run(self) -> Awaitable[None]: ...
    def run_blocking(self) -> None: ...

class TryxClient:
    async def send_message(self, chat: JID, message: MessageProto) -> str: ...

class Nu[T]:
    X: T
    pass

K = Nu[int]()
class Test:
    def on[L: int](self, event_type: Nu[L]) -> Callable[[Callable[[TryxClient, L], CoroutineType[None, None, Any| None]]], Callable[[TryxClient, L], CoroutineType[None, None, Any | None]]]: ...

