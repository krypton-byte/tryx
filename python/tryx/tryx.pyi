from typing import Any, Awaitable, Callable, Optional, Type

class JID:
    user: str
    server: str

    def __init__(self, user: str, server: str) -> None: ...

class Message:
    conversation: Optional[str]
    caption: Optional[str]
    message_info: MessageInfo

    def get_extended_text_message(self) -> Optional[str]: ...
    def get_text(self) -> Optional[str]: ...
    def raw_proto(self) -> Any: ...

class PairingQrCode:
    code: str
    timeout: int


class MessageInfo:
    id: str
    type: str
    push_name: str


class SqliteBackend:
    def __init__(self, path: str) -> None: ...

class Tryx:
    def __init__(self, backend: SqliteBackend) -> None: ...

    def on(self, event_type: Type[Any]) -> Callable[[Callable[..., Any]], Callable[..., Any]]: ...

    def run(self) -> Awaitable[None]: ...
    def run_blocking(self) -> None: ...
