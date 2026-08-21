"""Storage backend implementations for session persistence.

This module provides the storage backends that Tryx uses to persist
WhatsApp session data, device keys, and protocol state.

Supported backends:

- :class:`SqliteStore` — built-in SQLite backend (default, zero-config).
- ``FfiStoreProtocol`` — native FFI backend via C ABI (e.g. ``tryx-store-postgres``).
- ``StoreBase`` subclass — pure Python custom backend.

Example::

    from tryx.backend import SqliteStore

    backend = SqliteStore("whatsapp.db")
"""

from ._tryx import backend  # type: ignore

for name in dir(backend):  # type: ignore
    obj = getattr(backend, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

__all__ = [name for name in dir(backend) if isinstance(getattr(backend, name), type)]  # type: ignore
