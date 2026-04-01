"""Backend interfaces for Tryx runtime state storage."""

from abc import ABC

class BackendBase(ABC):
    """Base class for all backend implementations used by Tryx."""

class SqliteBackend(BackendBase):
    """SQLite-backed backend implementation.

    Use this backend to persist session and app-state data in a local file.
    """

    path: str

    def __init__(self, path: str) -> None:
        """Create a SQLite backend bound to a database file path."""
        ...
