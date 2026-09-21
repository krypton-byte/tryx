# :material-bell-ring: Event Model

Tryx uses an event-driven architecture. All WhatsApp interactions — messages,
presence changes, group updates, contact syncs — flow through typed event
classes. You register handlers for specific event types, and Tryx dispatches
them as they arrive.

## How Events Work

```
WhatsApp Stream → Rust Parser → Event Dispatcher → Your Handlers
```

1. **Raw protocol data** arrives over WebSocket
2. **Rust parser** normalizes it into typed event objects
3. **Event dispatcher** routes events to registered handlers
4. **Your handlers** process the event and respond

## Handler Registration

Handlers are registered using the `@app.on()` decorator:

```python
from tryx.client import Tryx
from tryx.events import EvMessage, EvConnected, EvDisconnected

app = Tryx(store)


@app.on(EvConnected)
async def on_connected(client):
    print("Connected to WhatsApp!")


@app.on(EvMessage)
async def on_message(client, event):
    print(f"Message from {event.sender}: {event.text}")


@app.on(EvDisconnected)
async def on_disconnected(client):
    print("Disconnected, will reconnect...")
```

### Handler Signature

Every handler receives two arguments:

```python
async def handler(client: TryxClient, event: EventType) -> None: ...
```

- `client` — The `TryxClient` instance, ready to send messages and query state
- `event` — The typed event object with all relevant data

### Multiple Handlers

You can register multiple handlers for the same event type. Both will be
called for each event:

```python
@app.on(EvMessage)
async def log_message(client, event):
    logger.info(f"Received: {event.message_id}")


@app.on(EvMessage)
async def process_message(client, event):
    if event.text:
        await handle_text(client, event)
```

---

## Event Taxonomy

### Lifecycle Events

Track the client connection lifecycle:

| Event | When | Use Case |
|-------|------|----------|
| `EvConnected` | WebSocket connected | Start sending messages |
| `EvDisconnected` | WebSocket lost | Pause non-critical operations |
| `EvLoggedOut` | Session invalidated | Re-authenticate |
| `EvStreamReplaced` | Another login replaced you | Check for duplicate sessions |
| `EvClientOutDated` | Client needs update | Prompt for upgrade |
| `EvReady` | Fully initialized | Safe to query state |

### Pairing Events

Track the phone pairing process:

| Event | When | Use Case |
|-------|------|----------|
| `EvPairingQrCode` | QR code generated | Display QR to user |
| `EvPairingCode` | Pairing code generated | Display code to user |
| `EvPairSuccess` | Pairing completed | Proceed with bot logic |
| `EvPairError` | Pairing failed | Show error, retry |

### Messaging Events

Track message flow:

| Event | When | Use Case |
|-------|------|----------|
| `EvMessage` | Message received | Process incoming messages |
| `EvReceipt` | Delivery/read receipt | Track message status |
| `EvUndecryptableMessage` | Decryption failed | Log and request re-send |
| `EvNotification` | Server notification | Handle system messages |

### Chat Action Sync Events

Track chat state changes (synced from other devices):

| Event | When | Use Case |
|-------|------|----------|
| `EvPinUpdate` | Chat pinned/unpinned | Update local state |
| `EvMuteUpdate` | Chat muted/unmuted | Adjust notifications |
| `EvArchiveUpdate` | Chat archived/unarchived | Update UI |
| `EvMarkChatAsReadUpdate` | Chat read status changed | Update unread count |
| `EvDeleteChatUpdate` | Chat deleted | Clean up local data |
| `EvDeleteMessageForMeUpdate` | Message deleted locally | Remove from cache |
| `EvStarUpdate` | Message starred/unstarred | Update star state |
| `EvChatArchive` | Chat archived | Update UI |

### Presence and Profile Events

Track user presence and profile changes:

| Event | When | Use Case |
|-------|------|----------|
| `EvPresence` | User typing/recording | Show typing indicator |
| `EvChatPresence` | Chat presence changed | Update UI |
| `EvAvailability` | Online/offline status | Track availability |
| `EvPicture` | Profile picture changed | Update avatar cache |
| `EvPushName` | Display name changed | Update contact info |
| `EvAbout` | Status text changed | Update status display |

### Contact and Device Sync Events

Track contact and device state:

| Event | When | Use Case |
|-------|------|----------|
| `EvContactUpdate` | Contact info changed | Update contact list |
| `EvContactNumberChanged` | Phone number changed | Update contact mapping |
| `EvDeviceListUpdate` | Device list changed | Re-key encryption |
| `EvGroupUpdate` | Group metadata changed | Update group info |
| `EvGroupInfoUpdate` | Group info refreshed | Sync group state |
| `EvJoinedGroup` | Bot joined a group | Initialize group state |
| `EvNewsletterUpdate` | Newsletter changed | Update channel info |
| `EvNewsletterLiveUpdate` | Live update received | Process live content |

### Business Events

| Event | When | Use Case |
|-------|------|----------|
| `EvBusinessStatusUpdate` | Business profile changed | Update business info |

---

## Event Payload Pattern

Many events expose a lazy `data` property:

```python
@app.on(EvMessage)
async def handle(client, event):
    # Access raw data (lazy-loaded from Rust)
    data = event.data

    # Repeated access reuses cached object
    same_data = event.data  # No re-parsing
```

The `data` property:

- Returns a rich typed object
- Converts from Rust internals on demand
- Caches the result for subsequent access

---

## Reliability Considerations

### Handler Execution

- Handlers are **async** and run on the tokio runtime
- Execution order is **event-driven**, not sequential
- Do not assume strict timing between different event types

### Error Handling

```python
@app.on(EvMessage)
async def safe_handler(client, event):
    try:
        await process_message(client, event)
    except Exception as e:
        logger.error(f"Handler failed: {e}")
        # Don't crash — Tryx will continue dispatching
```

!!! warning "Handler crashes"
    If a handler raises an exception, Tryx catches it and continues
    dispatching other events. However, the error is logged and may affect
    your bot's reliability.

### Idempotency

Build idempotent handlers using message identifiers:

```python
processed = set()


@app.on(EvMessage)
async def idempotent_handler(client, event):
    message_id = event.data.message_info.id
    if message_id in processed:
        return
    processed.add(message_id)
    await process_message(client, event)
```

### Ordering

Do not assume strict global ordering between all event types. For example:

- An `EvMessage` may arrive before its `EvReceipt`
- Group updates may arrive out of order
- Presence events may be stale

Design handlers to handle these cases gracefully.

---

## Event-to-Action Mapping

| Event | Typical Follow-up | Namespace |
|-------|-------------------|-----------|
| `EvMessage` | Reply, forward, react | Root send methods |
| `EvGroupUpdate` | Update local group state | [Groups](../api/groups.md) |
| `EvPresence` | Show typing indicator | [Presence](../api/presence.md) |
| `EvNewsletterLiveUpdate` | Process live updates | [Newsletter](../api/newsletter.md) |
| `EvContactUpdate` | Update contact list | [Contact](../api/contact.md) |
| `EvChatArchive` | Update UI state | [Chat Actions](../api/chat-actions.md) |
| `EvReceipt` | Track delivery status | Root send methods |
| `EvPicture` | Update avatar cache | [Contact](../api/contact.md) |

---

## Best Practices

1. **Keep handlers short** — Queue expensive work to background tasks
2. **Validate optional fields** — Check `event.text`, `event.media`, etc. before use
3. **Log metadata** — Include `jid`, `message_id`, timestamps for debugging
4. **Handle errors gracefully** — Don't let one bad event crash your bot
5. **Use typed events** — Prefer specific event classes over broad dynamic checks
6. **Treat undecryptable events as normal** — They happen during key rotation

---

## Related

- [Client API Gateway](../api/client.md) — namespace client reference
- [Reliability](../operations/reliability.md) — production error handling
- [Events API](../api/events.md) — full event type reference
