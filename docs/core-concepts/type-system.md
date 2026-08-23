# Type System

Tryx ships with complete `.pyi` stub files and a `py.typed` marker,
enabling full editor intelligence, static analysis, and API discoverability.

!!! tip "Why this matters"
    Typed contracts make event handling safer, API discovery faster, and
    refactors less risky. You get autocomplete for every method, parameter,
    and return type.

## How Typing Works

Tryx uses a two-layer typing approach:

1. **Runtime** — PyO3 generates Python classes from Rust structs
2. **Static** — `.pyi` stub files describe the exact API surface

```python
from tryx.client import Tryx, TryxClient
from tryx.types import JID, SendResult

# IDE shows: JID.whatsapp(phone: str) -> JID
jid = JID.whatsapp("5599800001")

# IDE shows: async def send_text(to: JID, text: str, ...) -> SendResult
result = await client.send_text(jid, "Hello!")

# IDE shows: result.message_id: str
print(result.message_id)
```

---

## Core Types

### JID — WhatsApp Address

The fundamental address type for all WhatsApp entities:

```python
from tryx.types import JID

# Personal account
jid = JID.whatsapp("5599800001")

# Group
group = JID.whatsapp_group("120363000000000000@g.us")

# Newsletter
newsletter = JID.whatsapp_newsletter("120363000000000000@newsletter")
```

### MessageInfo — Message Metadata

Contains identity, routing, and attribute information:

```python
from tryx.events import EvMessage


@app.on(EvMessage)
async def handle(client, event):
    info = event.data.message_info

    # Message identity
    message_id = info.message_id
    timestamp = info.timestamp

    # Routing
    sender = info.source.sender  # JID
    chat = info.source.chat  # JID
    participant = info.source.participant  # JID | None

    # Attributes
    msg_type = info.message_type  # "text", "image", etc.
    is_from_me = info.is_from_me
```

### SendResult — Send Operation Output

Returned by all send methods:

```python
result = await client.send_text(jid, "Hello")

result.message_id  # str: unique message ID
result.timestamp  # int: server timestamp
result.key  # MessageKey: message key for tracking
```

### MediaReuploadResult — Media Retry Output

Returned by `request_media_reupload`:

```python
result = await client.request_media_reupload(
    message_id="3EB0...",
    chat_jid=jid,
    media_key=b"...",
)

result.url  # str: re-uploaded media URL
result.direct_path  # str: direct download path
```

### UploadResponse — Media Upload Output

Returned by media upload methods:

```python
upload = await client.upload_photo(photo_bytes, jid)

upload.url  # str: media URL
upload.direct_path  # str: direct path
upload.media_key  # bytes: encryption key
upload.file_length  # int: file size
```

### ProfilePicture — Profile Image Metadata

```python
picture = await client.contact.get_profile_picture(jid, preview=False)

picture.url  # str: image URL
picture.direct_path  # str: direct download path
picture.file_length  # int: file size
picture.mimetype  # str: image MIME type
```

---

## Event Types

Every event class has a defined payload contract:

```python
from tryx.events import (
    EvMessage,  # Incoming message
    EvConnected,  # WebSocket connected
    EvDisconnected,  # WebSocket lost
    EvLoggedOut,  # Session invalidated
    EvPresence,  # User typing/recording
    EvGroupUpdate,  # Group metadata changed
    EvReceipt,  # Delivery/read receipt
)


@app.on(EvMessage)
async def handle(client, event: EvMessage):
    # event.sender: JID
    # event.text: str | None
    # event.media: MediaInfo | None
    # event.message_id: str
    # event.timestamp: int
    ...
```

No guessing with nested dict keys — everything is discoverable through
IDE autocomplete.

---

## Enum-Style Classes

Several Rust enums are exposed as Python classes with fixed attributes:

### Privacy

```python
from tryx.types import PrivacyCategory, PrivacyValue

# Categories
PrivacyCategory.LastSeen
PrivacyCategory.ProfilePhoto
PrivacyCategory.Status
PrivacyCategory.ReadReceipts
PrivacyCategory.Groups

# Values
PrivacyValue.Everyone
PrivacyValue.Contacts
PrivacyValue.Nobody
```

### Status

```python
from tryx.types import StatusPrivacySetting

StatusPrivacySetting.Contacts
StatusPrivacySetting.AllowList
StatusPrivacySetting.DenyList
```

### Chatstate

```python
from tryx.types import ChatStateType

ChatStateType.Composing  # User is typing
ChatStateType.Recording  # User is recording audio
ChatStateType.Paused  # User stopped typing
```

### Presence

```python
from tryx.types import PresenceStatus

PresenceStatus.Available
PresenceStatus.Unavailable
```

### Group Policies

```python
from tryx.types import MembershipApprovalMode, MemberAddMode, MemberLinkMode

MembershipApprovalMode.On
MembershipApprovalMode.Off

MemberAddMode.AdminOnly
MemberAddMode.AllMembers

MemberLinkMode.Admin
MemberLinkMode.AllMembers
```

### Newsletter

```python
from tryx.types import NewsletterVerification, NewsletterState, NewsletterRole

NewsletterVerification.Verified
NewsletterVerification.Unverified

NewsletterState.Active
NewsletterState.Suspended
NewsletterState.Geosuspended

NewsletterRole.Owner
NewsletterRole.Admin
NewsletterRole.Subscriber
NewsletterRole.Guest
```

### Events

```python
from tryx.types import EventResponse

EventResponse.Going
EventResponse.NotGoing
EventResponse.Maybe
```

---

## Type Boundary Pattern

Keep type boundaries clean between layers:

```python
from tryx.events import EvMessage
from tryx.types import JID


# Input boundary: accept typed events
def extract_sender(event: EvMessage) -> JID:
    return event.data.message_info.source.sender


# Output boundary: return typed results
async def forward_message(client, event: EvMessage, target: JID) -> SendResult:
    text = event.data.text or ""
    return await client.send_text(target, text)
```

---

## Static Analysis Workflow

### With Mypy

```bash
uv add mypy
uv run mypy your_project/
```

### With Pyright

```bash
uv add pyright
uv run pyright your_project/
```

### In CI

```yaml
# .github/workflows/ci.yml
- name: Type check
  run: |
    uv run mypy your_project/ --strict
    uv run pyright your_project/
```

---

## Best Practices

1. **Keep handler signatures explicit** — Always type-annotate `client` and `event`
2. **Annotate helper functions** — Especially those returning event-derived data
3. **Use specific event types** — `EvMessage` instead of generic `Event`
4. **Run static analysis in CI** — Catch type errors before deployment
5. **Leverage IDE autocomplete** — Let the stubs guide your API usage

---

## Related

- [Types API](../api/types.md) — full type reference
- [Events API](../api/events.md) — event type reference
- [Privacy Namespace](../api/privacy.md) — privacy type usage
- [Status Namespace](../api/status.md) — status type usage
