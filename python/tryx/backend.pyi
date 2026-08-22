"""Tryx Storage Backend API.

This module defines the storage backends supported by Tryx:

- **SqliteStore** — Built-in SQLite backend (default, zero-config).
- **FfiStore** — Native shared-library backend via C ABI (e.g. ``tryx-store-postgres``).
  Use the ``FfiStoreProtocol`` typing protocol for type-safe integration.
- **PythonStore** — Pure-Python async backend via inheritance from ``StoreBase``.
  Implement all abstract methods to create a custom store (e.g. Redis, MongoDB).

Complex Rust structs are serialized as JSON ``bytes`` across the FFI boundary.
Simple scalar types (``str``, ``int``, ``bool``) are passed natively.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Protocol, runtime_checkable

# ── Built-in backends ────────────────────────────────────────────────────────

class BackendBase:
    """Internal PyO3 base class for all store backends.

    End-users should **not** subclass this directly. Use one of:

    - :class:`SqliteStore` for the built-in SQLite backend.
    - :class:`StoreBase` for a custom pure-Python backend.
    - The ``FfiStoreProtocol`` protocol for native FFI-based backends.
    """

    ...

class SqliteStore(BackendBase):
    """Built-in SQLite storage backend.

    This is the default, zero-configuration backend. Data is persisted
    in a single ``*.db`` file using WAL mode.

    Args:
        path: Filesystem path to the SQLite database file.
              Created automatically if it doesn't exist.

    Example::

        from tryx.backend import SqliteStore

        backend = SqliteStore("whatsapp.db")
    """

    path: str

    def __init__(self, path: str) -> None: ...

# ── Internal Rust structs (JSON-serialized across FFI) ──────────────────────

class Device:
    """Cached device data (JSON-serialized across FFI)."""
    ...

class LidPnMappingEntry:
    """LID-to-phone-number mapping entry (JSON-serialized)."""
    ...

class DeviceListRecord:
    """Device list record (JSON-serialized)."""
    ...

class TcTokenEntry:
    """Trusted contact token entry (JSON-serialized)."""
    ...

class AppStateSyncKey:
    """App state sync key (JSON-serialized)."""
    ...

class HashState:
    """Hash state for app state sync (JSON-serialized)."""
    ...

class AppStateMutationMAC:
    """App state mutation MAC (JSON-serialized)."""
    ...

class MsgSecretEntry:
    """Message secret entry (JSON-serialized)."""
    ...

# ── FFI Store Protocol (for tryx-store-postgres, etc.) ───────────────────────

@runtime_checkable
class FfiStoreProtocol(Protocol):
    """Structural typing protocol for native FFI-based storage backends.

    Any object exposing a ``lib_path`` attribute and a ``connect_string``
    attribute satisfies this protocol without inheriting from anything
    in the Tryx package — keeping third-party store packages fully
    decoupled.

    The Tryx runtime loads the shared library (``*.so`` / ``*.dylib`` /
    ``*.dll``) at ``lib_path`` and calls standardized C-ABI entry points
    to perform storage operations with zero Python overhead.

    Example (tryx-store-postgres)::

        class PostgresStore:
            lib_path: str   # path to compiled .so
            connect_string: str

        backend = PostgresStore(
            lib_path="./libtryx_pg.so",
            connect_string="host=localhost dbname=tryx",
        )
    """

    lib_path: str
    connect_string: str

# ── Pure Python Store Base ───────────────────────────────────────────────────

class StoreBase(ABC):
    """Abstract base class for custom pure-Python storage backends.

    Subclass this and implement **all** abstract methods to create a custom
    backend using any async-capable database (Redis, MongoDB, DynamoDB, etc.).

    The Tryx Rust runtime detects ``StoreBase`` subclasses automatically
    via duck-typing and bridges each ``async def`` method through PyO3's
    async runtime.

    **Serialization convention:**

    - Simple types (``str``, ``int``, ``bool``) are passed as-is.
    - Complex structs are passed as **JSON-encoded bytes** and should
      be deserialized with ``json.loads(data)`` / serialized with
      ``json.dumps(obj).encode()``.

    **Performance notes:**

    - GIL is held only for the brief moment of calling into and
      extracting results from Python — the Rust side releases it
      during ``await``.
    - The bridge uses ``Python::attach`` (PyO3 0.28+) for minimal
      GIL acquisition overhead.

    Example::

        import json
        import redis.asyncio as redis
        from tryx.backend import StoreBase

        class RedisStore(StoreBase):
            def __init__(self, url: str = "redis://localhost"):
                self.r = redis.from_url(url)

            async def put_identity(self, address: str, key: bytes) -> None:
                await self.r.set(f"identity:{address}", key)

            async def load_identity(self, address: str) -> bytes | None:
                return await self.r.get(f"identity:{address}")

            # ... implement all other abstract methods ...
    """

    # ── SignalStore: Identity Operations ──────────────────────────────────

    @abstractmethod
    async def put_identity(self, address: str, key: bytes) -> None:
        """Store a 32-byte identity key for a remote address.

        Args:
            address: Signal address string (e.g. ``"5599800001@s.whatsapp.net"``).
            key: 32-byte identity public key.
        """
        ...

    @abstractmethod
    async def load_identity(self, address: str) -> bytes | None:
        """Load an identity key for a remote address.

        Returns:
            32 raw bytes of the identity key, or ``None`` if not found.
        """
        ...

    @abstractmethod
    async def delete_identity(self, address: str) -> None:
        """Delete an identity key."""
        ...

    # ── SignalStore: Session Operations ───────────────────────────────────

    @abstractmethod
    async def get_session(self, address: str) -> bytes | None:
        """Get an encrypted Signal session record.

        Returns:
            Opaque session bytes, or ``None`` if no session exists.
        """
        ...

    @abstractmethod
    async def put_session(self, address: str, session: bytes) -> None:
        """Store an encrypted Signal session record."""
        ...

    @abstractmethod
    async def delete_session(self, address: str) -> None:
        """Delete a Signal session."""
        ...

    # ── SignalStore: PreKey Operations ────────────────────────────────────

    @abstractmethod
    async def store_prekey(self, id: int, record: bytes, uploaded: bool) -> None:
        """Store a pre-key.

        Args:
            id: Pre-key ID (``u32``).
            record: Serialized pre-key record.
            uploaded: Whether this key has been uploaded to the server.
        """
        ...

    @abstractmethod
    async def load_prekey(self, id: int) -> bytes | None:
        """Load a pre-key by ID.

        Returns:
            Serialized pre-key record bytes, or ``None``.
        """
        ...

    @abstractmethod
    async def remove_prekey(self, id: int) -> None:
        """Remove a pre-key."""
        ...

    @abstractmethod
    async def get_max_prekey_id(self) -> int:
        """Get the maximum pre-key ID currently stored.

        Returns:
            The highest stored pre-key ID, or ``0`` if none exist.
        """
        ...

    # ── SignalStore: Signed PreKey Operations ─────────────────────────────

    @abstractmethod
    async def store_signed_prekey(self, id: int, record: bytes) -> None:
        """Store a signed pre-key."""
        ...

    @abstractmethod
    async def load_signed_prekey(self, id: int) -> bytes | None:
        """Load a signed pre-key by ID."""
        ...

    @abstractmethod
    async def load_all_signed_prekeys(self) -> list[tuple[int, bytes]]:
        """Load all signed pre-keys.

        Returns:
            List of ``(id, record_bytes)`` tuples.
        """
        ...

    @abstractmethod
    async def remove_signed_prekey(self, id: int) -> None:
        """Remove a signed pre-key."""
        ...

    # ── SignalStore: Sender Key Operations ────────────────────────────────

    @abstractmethod
    async def put_sender_key(self, address: str, record: bytes) -> None:
        """Store a sender key for group messaging."""
        ...

    @abstractmethod
    async def get_sender_key(self, address: str) -> bytes | None:
        """Get a sender key."""
        ...

    @abstractmethod
    async def delete_sender_key(self, address: str) -> None:
        """Delete a sender key."""
        ...

    # ── AppSyncStore ─────────────────────────────────────────────────────

    @abstractmethod
    async def get_sync_key(self, key_id: bytes) -> bytes | None:
        """Get an app state sync key by ID.

        Args:
            key_id: Raw key ID bytes.

        Returns:
            JSON-encoded ``AppStateSyncKey`` bytes, or ``None``.
        """
        ...

    @abstractmethod
    async def set_sync_key(self, key_id: bytes, key: bytes) -> None:
        """Set an app state sync key.

        Args:
            key_id: Raw key ID bytes.
            key: JSON-encoded ``AppStateSyncKey`` bytes.
        """
        ...

    @abstractmethod
    async def get_version(self, name: str) -> bytes:
        """Get the app state version for a collection.

        Args:
            name: Collection name (e.g. ``"critical_block"``).

        Returns:
            JSON-encoded ``HashState`` bytes.
        """
        ...

    @abstractmethod
    async def set_version(self, name: str, state: bytes) -> None:
        """Set the app state version for a collection.

        Args:
            state: JSON-encoded ``HashState`` bytes.
        """
        ...

    @abstractmethod
    async def put_mutation_macs(
        self, name: str, version: int, mutations: bytes
    ) -> None:
        """Store mutation MACs for a version.

        Args:
            version: App state version number (``u64``).
            mutations: JSON-encoded ``[AppStateMutationMAC]`` bytes.
        """
        ...

    @abstractmethod
    async def get_mutation_mac(self, name: str, index_mac: bytes) -> bytes | None:
        """Get a mutation MAC by index."""
        ...

    @abstractmethod
    async def delete_mutation_macs(self, name: str, index_macs: bytes) -> None:
        """Delete mutation MACs by their index MACs.

        Args:
            index_macs: JSON-encoded ``[Vec<u8>]`` bytes.
        """
        ...

    @abstractmethod
    async def get_latest_sync_key_id(self) -> bytes | None:
        """Get the most recently stored app state sync key ID."""
        ...

    # ── DeviceStore ──────────────────────────────────────────────────────

    @abstractmethod
    async def save(self, device: bytes) -> None:
        """Save device data.

        Args:
            device: JSON-encoded ``Device`` struct bytes.
        """
        ...

    @abstractmethod
    async def load(self) -> bytes | None:
        """Load device data.

        Returns:
            JSON-encoded ``Device`` bytes, or ``None`` if no device exists.
        """
        ...

    @abstractmethod
    async def exists(self) -> bool:
        """Check if a device exists in the store."""
        ...

    @abstractmethod
    async def create(self) -> int:
        """Create a new device row and return its generated device_id."""
        ...

    # ── ProtocolStore: Sender Key Device Tracking ────────────────────────

    @abstractmethod
    async def get_sender_key_devices(self, group_jid: str) -> list[tuple[str, bool]]:
        """Get sender key distribution status for all devices in a group.

        Returns:
            List of ``(device_jid, has_key)`` tuples.
        """
        ...

    @abstractmethod
    async def set_sender_key_status(self, group_jid: str, entries: bytes) -> None:
        """Set sender key status for devices.

        Args:
            entries: JSON-encoded ``[(&str, bool)]`` array bytes.
        """
        ...

    @abstractmethod
    async def clear_sender_key_devices(self, group_jid: str) -> None:
        """Clear all sender key device tracking for a group."""
        ...

    @abstractmethod
    async def delete_sender_key_device_rows(self, device_jids: bytes) -> None:
        """Delete specific sender_key_devices rows by device JID.

        Args:
            device_jids: JSON-encoded ``[str]`` array bytes.
        """
        ...

    @abstractmethod
    async def clear_all_sender_key_devices(self) -> None:
        """Clear all sender key device tracking across all groups."""
        ...

    # ── ProtocolStore: LID-PN Mapping ────────────────────────────────────

    @abstractmethod
    async def get_lid_mapping(self, lid: str) -> bytes | None:
        """Get a LID-to-phone-number mapping.

        Returns:
            JSON-encoded ``LidPnMappingEntry`` bytes, or ``None``.
        """
        ...

    @abstractmethod
    async def get_pn_mapping(self, phone: str) -> bytes | None:
        """Get a phone-number-to-LID mapping."""
        ...

    @abstractmethod
    async def put_lid_mapping(self, entry: bytes) -> None:
        """Store or update a LID-PN mapping.

        Args:
            entry: JSON-encoded ``LidPnMappingEntry`` bytes.
        """
        ...

    @abstractmethod
    async def get_all_lid_mappings(self) -> list[bytes]:
        """Get all LID-PN mappings.

        Returns:
            List of JSON-encoded ``LidPnMappingEntry`` bytes.
        """
        ...

    # ── ProtocolStore: Base Key Collision Detection ───────────────────────

    @abstractmethod
    async def save_base_key(
        self, address: str, message_id: str, base_key: bytes
    ) -> None:
        """Save a base key for retry collision detection."""
        ...

    @abstractmethod
    async def has_same_base_key(
        self, address: str, message_id: str, current_base_key: bytes
    ) -> bool:
        """Check if the current session has the same base key as the saved one."""
        ...

    @abstractmethod
    async def delete_base_key(self, address: str, message_id: str) -> None:
        """Delete a base key entry."""
        ...

    # ── ProtocolStore: Device Registry ───────────────────────────────────

    @abstractmethod
    async def update_device_list(self, record: bytes) -> None:
        """Update the device list for a user.

        Args:
            record: JSON-encoded ``DeviceListRecord`` bytes.
        """
        ...

    @abstractmethod
    async def get_devices(self, user: str) -> bytes | None:
        """Get all known devices for a user.

        Returns:
            JSON-encoded ``DeviceListRecord`` bytes, or ``None``.
        """
        ...

    @abstractmethod
    async def delete_devices(self, user: str) -> None:
        """Delete a device list record."""
        ...

    # ── ProtocolStore: TcToken ───────────────────────────────────────────

    @abstractmethod
    async def get_tc_token(self, jid: str) -> bytes | None:
        """Get a trusted contact token.

        Returns:
            JSON-encoded ``TcTokenEntry`` bytes, or ``None``.
        """
        ...

    @abstractmethod
    async def put_tc_token(self, jid: str, entry: bytes) -> None:
        """Store or update a trusted contact token.

        Args:
            entry: JSON-encoded ``TcTokenEntry`` bytes.
        """
        ...

    @abstractmethod
    async def delete_tc_token(self, jid: str) -> None:
        """Delete a trusted contact token."""
        ...

    @abstractmethod
    async def get_all_tc_token_jids(self) -> list[str]:
        """Get all JIDs that have stored tc tokens."""
        ...

    @abstractmethod
    async def delete_expired_tc_tokens(
        self, token_cutoff: int, sender_cutoff: int
    ) -> int:
        """Delete tc tokens whose received token and sender bucket are both expired.

        A row is removed only when its received token is expired-or-absent
        (older than ``token_cutoff``) AND its sender bucket is expired-or-absent
        (older than ``sender_cutoff``), so recent state on one axis keeps the row.

        Args:
            token_cutoff: Unix timestamp in seconds; received tokens older than
                this are considered expired.
            sender_cutoff: Unix timestamp in seconds; sender buckets older than
                this are considered expired.

        Returns:
            Number of rows deleted.
        """
        ...

    # ── ProtocolStore: Sent Message Store ─────────────────────────────────

    @abstractmethod
    async def store_sent_message(
        self, chat_jid: str, message_id: str, payload: bytes
    ) -> None:
        """Store a sent message's serialized payload for retry handling."""
        ...

    @abstractmethod
    async def take_sent_message(self, chat_jid: str, message_id: str) -> bytes | None:
        """Retrieve and delete a sent message (atomic take)."""
        ...

    @abstractmethod
    async def delete_expired_sent_messages(self, cutoff: int) -> int:
        """Delete sent messages older than cutoff.

        Returns:
            Number of rows deleted.
        """
        ...

    # ── MsgSecretStore ───────────────────────────────────────────────────

    @abstractmethod
    async def put_msg_secrets(self, entries: bytes) -> int:
        """Batch-upsert message secrets.

        Args:
            entries: JSON-encoded ``[MsgSecretEntry]`` bytes.

        Returns:
            Number of rows affected.
        """
        ...

    @abstractmethod
    async def get_msg_secret(self, chat: str, sender: str, msg_id: str) -> bytes | None:
        """Fetch the persisted message secret.

        Returns:
            Raw secret bytes, or ``None`` if absent.
        """
        ...

    @abstractmethod
    async def delete_expired_msg_secrets(self, cutoff: int) -> int:
        """Delete expired message secrets.

        Returns:
            Number of rows deleted.
        """
        ...
