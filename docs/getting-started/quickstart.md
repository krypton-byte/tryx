# Quick Start

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

## Runtime Flow

1. Create backend storage.
2. Create `Tryx` bot instance.
3. Register handlers with `@bot.on(EventClass)`.
4. Start runtime with `await bot.run()`.
5. Use `TryxClient` inside handlers for API calls.

## Blocking Script Mode

For quick scripts without manual event loop management:

```python
from tryx.backend import SqliteBackend
from tryx.client import Tryx

bot = Tryx(SqliteBackend("whatsapp.db"))
bot.run_blocking()
```

## Next Steps

- Read [Authentication Flow](authentication.md) to understand pairing and session persistence.
- Explore [Client API](../api/client.md) for all namespace methods.
- Review [Event Model](../core-concepts/event-model.md) before building complex logic.
