from abc import ABC

class BackendBase(ABC):
    ...

class SqliteBackend(BackendBase):
    path: str

    def __init__(self, path: str) -> None: ...
