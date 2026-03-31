# Quick Start

Build and run a minimal echo bot, then expand it safely.

!!! tip "Expected outcome"
    You should receive incoming text and reply with an echo message in the same chat.

## Minimal Bot

```python
import asyncio

from tryx.backend import SqliteBackend
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage
from tryx.waproto.whatsapp_pb2 import Message

backend = SqliteBackend("whatsapp.db")
bot = Tryx(backend)


@bot.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = event.data.get_text() or "<non-text>"
    chat = event.data.message_info.source.chat
    await client.send_message(chat, Message(conversation=f"Echo: {text}"))


async def main() -> None:
    await bot.run()


if __name__ == "__main__":
    asyncio.run(main())
```

## How It Works

1. backend persists pairing/session state
2. `Tryx` runtime wires event dispatcher
3. `@bot.on(EvMessage)` registers handler
4. `TryxClient` executes namespace/root API calls

## Runtime Flow

1. Create backend storage.
2. Create `Tryx` bot instance.
3. Register handlers with `@bot.on(EventClass)`.
4. Start runtime with `await bot.run()`.
5. Use `TryxClient` inside handlers for API calls.

## First Production Hardening

=== "Reliability"
    - deduplicate with message id
    - bound retries for network operations

=== "Safety"
    - validate command input
    - keep admin-only commands restricted

=== "Performance"
    - keep handlers short
    - offload heavy work to worker queue

## Blocking Script Mode

For quick scripts without manual event loop management:

```python
from tryx.backend import SqliteBackend
from tryx.client import Tryx

bot = Tryx(SqliteBackend("whatsapp.db"))
bot.run_blocking()
```

!!! warning
    `run_blocking()` is convenient for small scripts. Prefer explicit async runtime control for larger systems.

## Next Steps

- Read [Authentication Flow](authentication.md) to understand pairing and session persistence.
- Explore [Client API Gateway](../api/client.md) for all namespace methods.
- Review [Event Model](../core-concepts/event-model.md) before building complex logic.
- Continue with [Tutorial: Command Bot](../tutorials/command-bot.md).
