# Type System

Tryx ships with `.pyi` stubs and `py.typed`, enabling full editor and type-checker support.

## Core Types

- `JID`: canonical address object
- `MessageSource`: message origin and routing context
- `MessageInfo`: metadata for message identity and attributes
- `UploadResponse`: media upload output
- `SendResult`: send operation result
- `MediaReuploadResult`: media retry result
- `ProfilePicture`: profile picture metadata

## Event Types

Event classes define explicit payload contracts:

- no guessing with nested dict keys
- discoverable through IDE autocomplete
- easier static checks in large projects

## Enum-Style Classes

Several Rust enums are exposed as Python classes with fixed attributes (for example status/privacy and event reason classes).

## Suggested Typing Workflow

1. Keep handler function signatures explicit.
2. Annotate helper functions returning event-derived data.
3. Run static analysis in CI (mypy or pyright).

## Example

```python
from tryx.events import EvMessage
from tryx.types import JID


def extract_chat(event: EvMessage) -> JID:
    return event.data.message_info.source.chat
```
