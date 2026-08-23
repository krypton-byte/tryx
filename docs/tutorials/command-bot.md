# Tutorial: Command Bot

Build a command-driven WhatsApp bot that stays maintainable as command count
grows. This tutorial progresses from a minimal router to a production-ready
pattern with error handling, idempotency, and logging.

!!! tip "Outcome"
    At the end of this tutorial you will have:
    - A clean command parser with table-driven dispatch
    - Production-safe error handling and idempotency guards
    - Structured logging for debugging

---

## Level 1: Basic Command Router

```python
import asyncio
from tryx.client import Tryx
from tryx.backend import SqliteStore
from tryx.events import EvMessage

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)


def normalize(text: str | None) -> str:
    return (text or "").strip().lower()


@app.on(EvMessage)
async def on_message(client, event):
    text = normalize(event.data.get_text())
    chat = event.data.message_info.source.chat

    if text == "ping":
        await client.send_text(chat, "pong", quoted=event)
    elif text == "help":
        await client.send_text(chat, "commands: ping, help", quoted=event)


asyncio.run(app.run())
```

**How it works:**

1. `normalize()` strips whitespace and lowercases for consistent matching
2. `@app.on(EvMessage)` fires for every incoming message
3. `quoted=event` replies to the original message (thread-like UX)

---

## Level 2: Table-Driven Commands

For bots with many commands, use a dispatch table:

```python
from collections.abc import Awaitable, Callable
from tryx.client import Tryx
from tryx.backend import SqliteStore
from tryx.events import EvMessage

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)

CommandHandler = Callable[[TryxClient, EvMessage, list[str]], Awaitable[None]]


async def cmd_ping(client, event, args):
    chat = event.data.message_info.source.chat
    await client.send_text(chat, "pong", quoted=event)


async def cmd_echo(client, event, args):
    chat = event.data.message_info.source.chat
    await client.send_text(chat, " ".join(args) or "(empty)", quoted=event)


async def cmd_help(client, event, args):
    chat = event.data.message_info.source.chat
    commands = ", ".join(COMMANDS.keys())
    await client.send_text(chat, f"Commands: {commands}", quoted=event)


COMMANDS: dict[str, CommandHandler] = {
    "ping": cmd_ping,
    "echo": cmd_echo,
    "help": cmd_help,
}


@app.on(EvMessage)
async def on_command(client, event):
    text = (event.data.get_text() or "").strip()
    if not text.startswith("/"):
        return

    parts = text[1:].split()
    name = parts[0].lower()
    args = parts[1:]

    fn = COMMANDS.get(name)
    if fn is None:
        chat = event.data.message_info.source.chat
        await client.send_text(chat, f"Unknown command: {name}")
        return

    await fn(client, event, args)
```

**Benefits:**

- Adding a command = adding one async function + one dict entry
- Command parsing is separated from side effects
- Easy to test individual command handlers

---

## Level 3: Production Pattern

### Idempotent Dispatch

Prevent duplicate processing from reconnections or retries:

```python
seen_ids: set[str] = set()


@app.on(EvMessage)
async def on_idempotent(client, event):
    message_id = event.data.message_info.id
    if message_id in seen_ids:
        return
    seen_ids.add(message_id)

    # dispatch command here
```

!!! warning "Memory growth"
    If you store processed IDs in memory, add TTL eviction or persist
    compact dedupe state. For production, consider a bounded set or
    Redis-backed deduplication.

### Structured Logging

```python
import logging

logger = logging.getLogger("bot")


@app.on(EvMessage)
async def on_logged(client, event):
    message_id = event.data.message_info.id
    chat = event.data.message_info.source.chat
    text = event.data.get_text() or ""

    logger.info(
        "message_received",
        extra={
            "message_id": message_id,
            "chat": str(chat),
            "text_preview": text[:50],
        },
    )

    # ... process command
```

### Error Recovery

```python
@app.on(EvMessage)
async def on_safe(client, event):
    try:
        await process_command(client, event)
    except ValueError as e:
        chat = event.data.message_info.source.chat
        await client.send_text(chat, f"Error: {e}")
    except Exception as e:
        logger.error(f"Handler failed: {e}", exc_info=True)
        # Don't crash — Tryx continues dispatching
```

---

## Level 4: Advanced Features

### Typing Indicators

Show typing while processing:

```python
@app.on(EvMessage)
async def with_typing(client, event):
    chat = event.data.message_info.source.chat

    # Show "typing..." while processing
    await client.chatstate.send_composing(chat)

    # Simulate processing time
    result = await expensive_operation()
    await client.send_text(chat, f"Result: {result}", quoted=event)
```

### Admin-Only Commands

```python
ADMIN_JIDS = {
    JID.whatsapp("5599800001"),
    JID.whatsapp("5599800002"),
}


async def cmd_ban(client, event, args):
    chat = event.data.message_info.source.chat
    sender = event.data.message_info.source.sender

    if sender not in ADMIN_JIDS:
        await client.send_text(chat, "Admin only")
        return

    if not args:
        await client.send_text(chat, "Usage: /ban <phone>")
        return

    target = JID.whatsapp(args[0])
    await client.blocking.block(target)
    await client.send_text(chat, f"Blocked {args[0]}")
```

### Command Aliases

```python
COMMANDS = {
    "ping": cmd_ping,
    "p": cmd_ping,  # alias
    "echo": cmd_echo,
    "e": cmd_echo,  # alias
    "help": cmd_help,
    "h": cmd_help,  # alias
    "?": cmd_help,  # alias
}
```

---

## Production Example: Full Bot

```python
import asyncio
import logging
from tryx.client import Tryx, TryxClient
from tryx.backend import SqliteStore
from tryx.events import EvMessage, EvConnected

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("bot")

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)

seen_ids: set[str] = set()


@app.on(EvConnected)
async def on_connected(client):
    logger.info("Bot connected to WhatsApp")


@app.on(EvMessage)
async def on_message(client, event):
    message_id = event.data.message_info.id

    # Idempotency guard
    if message_id in seen_ids:
        return
    seen_ids.add(message_id)

    text = (event.data.get_text() or "").strip()
    if not text.startswith("/"):
        return

    parts = text[1:].split()
    name = parts[0].lower()
    args = parts[1:]

    fn = COMMANDS.get(name)
    if fn is None:
        return

    try:
        await fn(client, event, args)
    except Exception as e:
        logger.error(f"Command {name} failed: {e}", exc_info=True)
        chat = event.data.message_info.source.chat
        await client.send_text(chat, f"Error: {e}")


asyncio.run(app.run())
```

---

## Where to Go Next

- [Chat Actions Namespace](../api/chat-actions.md) — edit, revoke, react, archive
- [Profile and Privacy Tutorial](profile-privacy.md) — manage profile settings
- [Reliability Operations](../operations/reliability.md) — production error handling
- [Media Workflows](media-workflows.md) — send photos, audio, video
