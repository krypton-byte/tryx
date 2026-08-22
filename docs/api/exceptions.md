# Exceptions API

::: tryx.exceptions.FailedBuildClient
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.exceptions.FailedToDecodeProto
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.exceptions.EventDispatchError
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.exceptions.PyPayloadBuildError
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.exceptions.UnsupportedBackend
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.exceptions.UnsupportedEventType
    options:
      show_root_heading: true
      heading_level: 2

---

Exception hierarchy for Tryx error handling.

## Exception Hierarchy

```
Exception
├── FailedBuildClient     — Client initialization failure
│   ├── BuildBotError     — (legacy alias)
│   └── FailedBuildBot    — (legacy alias)
├── FailedToDecodeProto   — Protobuf decoding failure
├── EventDispatchError    — Event callback dispatch failure
├── PyPayloadBuildError   — Python-to-Rust payload conversion failure
├── UnsupportedBackend    — Incompatible storage backend
│   └── UnsupportedBackendError — (legacy alias)
└── UnsupportedEventType  — Unknown event class registration
    └── UnsupportedEventTypeError — (legacy alias)
```

## Usage Examples

### Client Initialization

```python
from tryx.exceptions import FailedBuildClient

try:
    app = Tryx(backend)
except FailedBuildClient as e:
    print(f"Failed to initialize client: {e}")
```

### Event Handling

```python
from tryx.exceptions import EventDispatchError


@app.on(EvMessage)
async def on_message(client, event):
    try:
        # process event
        pass
    except EventDispatchError as e:
        print(f"Dispatch failed: {e}")
```

### Backend Errors

```python
from tryx.exceptions import UnsupportedBackend

if not isinstance(backend, (SqliteStore, FfiStoreProtocol, StoreBase)):
    raise UnsupportedBackend(f"Unsupported backend type: {type(backend)}")
```

## Backward Compatibility

Legacy exception names are available as aliases:

| Legacy Name | Modern Name |
|-------------|-------------|
| `BuildBotError` | `FailedBuildClient` |
| `FailedBuildBot` | `FailedBuildClient` |
| `UnsupportedEventTypeError` | `UnsupportedEventType` |
| `UnsupportedBackendError` | `UnsupportedBackend` |

!!! warning "Migration"
    If you're using legacy exception names, update to the modern names.
    Legacy aliases will be removed in a future major version.
