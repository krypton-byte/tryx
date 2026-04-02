# Tutorial: Command Automation

Build a command-driven automation that stays maintainable as command count grows.

!!! tip "Outcome"
    At the end of this tutorial you will have:
    - clean command parser
    - command dispatch table
    - production-safe error handling and idempotency guard

## Level 1: Basic Command Router

```python
import asyncio

from tryx.backend import SqliteBackend
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage

backend = SqliteBackend("whatsapp.db")
app = Tryx(backend)


def normalize(text: str | None) -> str:
    return (text or "").strip().lower()


@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = normalize(event.data.get_text())
    chat = event.data.message_info.source.chat

    if text == "ping":
        await client.send_text(chat, "pong", quoted=event)
    elif text == "help":
        await client.send_text(chat, "commands: ping, help", quoted=event)


asyncio.run(app.run())
```

## Level 2: Table-driven Commands

```python
from collections.abc import Awaitable, Callable

CommandHandler = Callable[[TryxClient, EvMessage, list[str]], Awaitable[None]]


async def cmd_ping(client: TryxClient, event: EvMessage, args: list[str]) -> None:
    chat = event.data.message_info.source.chat
    await client.send_text(chat, "pong", quoted=event)


async def cmd_echo(client: TryxClient, event: EvMessage, args: list[str]) -> None:
    chat = event.data.message_info.source.chat
    await client.send_text(chat, " ".join(args) or "(empty)", quoted=event)


COMMANDS: dict[str, CommandHandler] = {
    "ping": cmd_ping,
    "echo": cmd_echo,
}


@app.on(EvMessage)
async def on_command(client: TryxClient, event: EvMessage) -> None:
    text = (event.data.get_text() or "").strip()
    if not text.startswith("/"):
        return

    parts = text[1:].split()
    name, args = parts[0].lower(), parts[1:]
    fn = COMMANDS.get(name)
    if fn is None:
        await client.send_text(event.data.message_info.source.chat, f"Unknown command: {name}")
        return
    await fn(client, event, args)
```

## Level 3: Production Pattern

=== "Reliability"
    - Add per-message idempotency key from `event.data.message_info.id`.
    - Separate command parsing from side effects.
    - Add structured logging (`command`, `chat`, `message_id`).

=== "Safety"
    - Use allowlist for admin-only commands.
    - Validate argument length/type before action.
    - Wrap outbound mutations with retry policy for transient failures.

=== "UX"
    - Send `client.chatstate.send_composing(chat)` during slow command execution.
    - Return explicit error message for invalid command arguments.

## Production Example: Idempotent Dispatch

```python
seen_ids: set[str] = set()


@app.on(EvMessage)
async def on_idempotent(client: TryxClient, event: EvMessage) -> None:
    message_id = event.data.message_info.id
    if message_id in seen_ids:
        return
    seen_ids.add(message_id)

    # dispatch command here
```

??? info "Advanced: plugin command registry"
    For larger bots, store command handlers in module-level plugins and register them into a central command registry at startup.
    This keeps each command domain isolated and testable.

!!! warning "Memory growth"
    If you store processed IDs in memory, add TTL eviction or persist compact dedupe state.

## Where To Go Next

- [Chat Actions Namespace](../api/chat-actions.md)
- [Profile and Privacy Tutorial](profile-privacy.md)
- [Reliability Operations](../operations/reliability.md)
