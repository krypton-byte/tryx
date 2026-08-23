# Storage Backend API

::: tryx.backend.SqliteStore
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.backend.FfiStoreProtocol
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.backend.StoreBase
    options:
      show_root_heading: true
      heading_level: 2

---

This module defines the storage backends supported by Tryx for session
persistence and device key management.

## Backend Tiers

| Backend | Type | Use Case |
|---------|------|----------|
| `SqliteStore` | Built-in | Default, zero-config, single-file storage |
| `FfiStoreProtocol` | Native FFI | High-performance backends (PostgreSQL, MySQL) |
| `StoreBase` | Pure Python | Custom backends (Redis, MongoDB, etc.) |

## SqliteStore

The default backend. Data is persisted in a single `*.db` file using WAL mode.

```python
from tryx.backend import SqliteStore

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)
```

## FfiStoreProtocol

Structural typing protocol for native FFI-based storage backends. Any object
exposing `lib_path` and `config_json` attributes satisfies this protocol.

```python
import json

class PostgresStore:
    lib_path: str  # path to compiled .so
    config_json: str


backend = PostgresStore(
    lib_path="./libtryx_pg.so",
    config_json=json.dumps({"host": "localhost", "dbname": "tryx"}),
)
```

## StoreBase (Custom Python Backend)

Inherit from `StoreBase` to create a pure-Python async backend. Implement all
abstract methods for full control over storage operations.

```python
from tryx.backend import StoreBase


class RedisStore(StoreBase):
    async def get(self, key: str) -> bytes | None: ...

    async def set(self, key: str, value: bytes) -> None: ...

    async def delete(self, key: str) -> None: ...
```

## When to Choose Each Backend

=== "SqliteStore"
    - Single-user bots
    - Development and testing
    - Embedded applications
    - No external dependencies

=== "FfiStoreProtocol"
    - Production deployments
    - Multi-user systems
    - High-throughput requirements
    - PostgreSQL/MySQL needed

=== "StoreBase"
    - Redis/MongoDB/DynamoDB
    - Custom caching layers
    - Cloud-native storage
    - Experimental backends

!!! tip "Migration path"
    Start with `SqliteStore` for development, then migrate to `FfiStoreProtocol`
    or `StoreBase` for production. The API surface is identical.
