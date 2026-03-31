# Tutorial: Command Bot

## Goal

Build a command-based bot with clean handler routing.

## Example Skeleton

```python
import asyncio

from tryx.backend import SqliteBackend
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage

backend = SqliteBackend("whatsapp.db")
bot = Tryx(backend)


def normalize(text: str | None) -> str:
    return (text or "").strip().lower()


@bot.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = normalize(event.data.get_text())
    chat = event.data.message_info.source.chat

    if text == "ping":
        await client.send_text(chat, "pong", quoted=event)
    elif text == "help":
        await client.send_text(chat, "commands: ping, help", quoted=event)


asyncio.run(bot.run())
```

## Production Tips

- Keep command parsing pure and testable.
- Avoid large logic branches inside one callback.
- Split commands into dedicated async functions.
