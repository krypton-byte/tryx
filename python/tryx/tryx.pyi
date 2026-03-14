from types import CoroutineType
from typing import Any, Awaitable, Callable, Type
from .waproto.whatsapp_pb2 import Message as MessageProto
from .types import JID

class SqliteBackend:
    def __init__(self, path: str) -> None: ...

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

class TryxClient:
    async def send_message(self, chat: JID, message: MessageProto) -> str: ...
