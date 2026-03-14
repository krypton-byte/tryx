from abc import ABC


class BackendBase(ABC):
    pass


class SqliteBackend(BackendBase):
    def __init__(self, path: str) -> None: ...
    @property
    def path(self) -> str: ...