# Quick Start

Build a working WhatsApp bot in 5 minutes. This guide walks through the
minimal setup, then expands with real functionality.

## Step 1: Create Your Bot

Create a file called `bot.py`:

```python
import asyncio
from tryx.client import Tryx
from tryx.backend import SqliteStore
from tryx.events import EvMessage

# Create storage backend (persists session and protocol state)
backend = SqliteStore("whatsapp.db")

# Initialize the Tryx runtime
app = Tryx(backend)


# Register an event handler
@app.on(EvMessage)
async def on_message(client, event):
    text = event.data.get_text() or ""
    chat = event.data.message_info.source.chat

    if text.lower() == "ping":
        await client.send_text(chat, "pong")


# Start the bot
asyncio.run(app.run())
```

## Step 2: Run and Pair

```bash
python bot.py
```

On first run, Tryx will emit a pairing event. Scan the QR code with your
WhatsApp mobile app (Linked Devices → Link a Device).

!!! info "Pairing modes"
    - **QR code**: Scan with your phone camera
    - **Numeric code**: Enter the 8-digit code on your phone

After pairing, the session is stored in `whatsapp.db` and you won't need
to pair again.

## Step 3: Test It

Send "ping" to your WhatsApp number from another device. The bot should
reply with "pong".

---

## How It Works

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  SqliteStore │────▶│     Tryx     │────▶│  WebSocket   │
│  (persist)   │     │  (dispatcher)│     │  (WhatsApp)  │
└──────────────┘     └──────┬───────┘     └──────────────┘
                            │
                     ┌──────▼───────┐
                     │  Your Handler│
                     │  @app.on()   │
                     └──────────────┘
```

1. **`SqliteStore`** creates and manages a local database for session
   persistence and Signal protocol state.
2. **`Tryx`** initializes the Rust runtime and event dispatcher.
3. **`@app.on(EvMessage)`** registers your handler to fire on every
   incoming message.
4. **`await app.run()`** opens the WebSocket connection and begins
   processing events.

---

## Sending Messages

### Text

```python
from tryx.types import JID

jid = JID.whatsapp("5599800001")
await client.send_text(jid, "Hello from Tryx!")
```

### Photo with Caption

```python
with open("photo.jpg", "rb") as f:
    photo_data = f.read()

result = await client.send_photo(
    to=jid,
    photo_data=photo_data,
    caption="Check this out!",
)
print(f"Sent: {result.message_id}")
```

### Voice Message

```python
with open("voice.ogg", "rb") as f:
    audio_data = f.read()

await client.send_audio(
    to=jid,
    audio_data=audio_data,
    ptt=True,  # Push-to-talk (voice note)
    seconds=15,  # Duration hint
)
```

### Document

```python
with open("report.pdf", "rb") as f:
    doc_data = f.read()

await client.send_document(
    to=jid,
    document_data=doc_data,
    file_name="report.pdf",
    caption="Monthly report",
)
```

### Video

```python
with open("clip.mp4", "rb") as f:
    video_data = f.read()

await client.send_video(
    to=jid,
    video_data=video_data,
    caption="Check this clip",
)
```

### Sticker

```python
with open("sticker.webp", "rb") as f:
    sticker_data = f.read()

await client.send_sticker(to=jid, sticker_data=sticker_data)
```

---

## Handling Events

### Multiple Event Types

```python
from tryx.events import EvMessage, EvConnected, EvDisconnected


@app.on(EvConnected)
async def on_connected(client):
    print("Connected to WhatsApp!")


@app.on(EvMessage)
async def on_message(client, event):
    text = event.data.get_text() or ""
    chat = event.data.message_info.source.chat

    if text == "!ping":
        await client.send_text(chat, "Pong!")
    elif text == "!info":
        stats = await client.advanced.stats()
        await client.send_text(chat, f"Stats: {stats}")
    elif text == "!groups":
        groups = await client.groups.get_participating()
        names = [m.subject for m in groups.values()]
        await client.send_text(chat, f"Groups: {', '.join(names)}")


@app.on(EvDisconnected)
async def on_disconnected(client):
    print("Disconnected, will reconnect...")
```

### Processing Media

```python
@app.on(EvMessage)
async def handle_media(client, event):
    chat = event.data.message_info.source.chat

    if event.data.media:
        media_type = event.data.media.media_type

        if media_type == "image":
            await client.send_text(chat, "Nice photo!")
        elif media_type == "video":
            await client.send_text(chat, "Cool video!")
        elif media_type == "audio":
            await client.send_text(chat, "Got your audio!")
```

### Quoting Messages

```python
@app.on(EvMessage)
async def handle_reply(client, event):
    text = event.data.get_text() or ""
    chat = event.data.message_info.source.chat

    if text.lower() == "ping":
        # Quote the original message
        await client.send_text(chat, "pong", quoted=event)
```

---

## Error Handling

Always wrap handler logic to prevent one bad event from crashing your bot:

```python
import logging

logger = logging.getLogger(__name__)


@app.on(EvMessage)
async def safe_handler(client, event):
    try:
        text = event.data.get_text() or ""
        chat = event.data.message_info.source.chat

        if text:
            await client.send_text(chat, f"Echo: {text}")
    except Exception as e:
        logger.error(f"Handler failed: {e}")
        # Don't crash — Tryx continues dispatching
```

!!! warning "Handler exceptions"
    If a handler raises an exception, Tryx catches it and continues
    dispatching other events. However, the error is logged and may affect
    your bot's reliability. Always handle errors explicitly.

---

## Blocking Mode

For quick scripts without manual event loop management:

```python
from tryx.backend import SqliteStore
from tryx.client import Tryx

app = Tryx(SqliteStore("whatsapp.db"))
app.run_blocking()
```

!!! tip "When to use"
    `run_blocking()` is convenient for small scripts and prototyping.
    For larger systems, prefer explicit `asyncio.run(app.run())`.

---

## Production Tips

=== "Reliability"

    - Deduplicate with `event.data.message_info.id`
    - Bound retries for network operations
    - Handle undecryptable messages gracefully

=== "Safety"

    - Validate command input
    - Restrict admin-only commands
    - Rate-limit expensive operations

=== "Performance"

    - Keep handlers short and async
    - Offload heavy work to background tasks
    - Use connection pooling for databases

---

## Next Steps

- [Authentication Flow](authentication.md) — understand pairing and session persistence
- [Architecture](../core-concepts/architecture.md) — how Tryx works internally
- [Client API Gateway](../api/client.md) — all namespace methods
- [Tutorial: Command Bot](../tutorials/command-bot.md) — build a real bot
