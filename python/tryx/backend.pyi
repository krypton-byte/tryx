"""Backend interfaces for Tryx runtime state storage."""

from abc import ABC
from typing import Optional

class BackendBase(ABC):
    """Base class for all backend implementations used by Tryx."""

class SqliteStore(BackendBase):
    """SQLite-backed storage implementation.

    Use this backend to persist session and app-state data in a local file.
    """

    path: str

    def __init__(self, path: str) -> None:
        """Create a SQLite storage backend bound to a database file path."""
        ...


from typing import Protocol, runtime_checkable

@runtime_checkable
class FfiStoreProtocol(Protocol):
    """
    Protocol defining an external FFI store via duck typing.
    Any Python object with `lib_path` and `config_json` attributes
    will be dynamically accepted by Tryx as an FFI store backend.
    """
    @property
    def lib_path(self) -> str: ...
    @property
    def config_json(self) -> str: ...
