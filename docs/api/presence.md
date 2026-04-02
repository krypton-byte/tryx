# Presence Namespace (`client.presence`)

`client.presence` publishes your presence state and subscribes to contact presence updates.

## Methods

| Method | Purpose |
| --- | --- |
| `set(status)` | Explicitly set `PresenceStatus` |
| `set_available()` | Shortcut to available |
| `set_unavailable()` | Shortcut to unavailable |
| `subscribe(jid)` | Subscribe to one user's presence |
| `unsubscribe(jid)` | Stop receiving presence updates |

## Presence Status Values

- `PresenceStatus.Available`
- `PresenceStatus.Unavailable`

## Runnable Example: Presence Monitoring

```python
from tryx.events import EvMessage
from tryx.types import JID


@app.on(EvMessage)
async def monitor_presence(client, event):
    text = (event.data.get_text() or "").strip()
    chat = event.data.message_info.source.chat

    if text.startswith("/presence watch "):
        phone = text.split(maxsplit=2)[2]
        target = JID(user=phone, server="s.whatsapp.net")
        await client.presence.subscribe(target)
        await client.send_text(chat, f"Subscribed to {phone} presence")
    elif text.startswith("/presence unwatch "):
        phone = text.split(maxsplit=2)[2]
        target = JID(user=phone, server="s.whatsapp.net")
        await client.presence.unsubscribe(target)
        await client.send_text(chat, f"Unsubscribed {phone}")
```

## Operational Guidance

=== "Startup"
    Set your own baseline status explicitly on service start.

    ```python
    await client.presence.set_available()
    ```

=== "Shutdown"
    Mark unavailable during graceful shutdown to reduce stale online state.

    ```python
    await client.presence.set_unavailable()
    ```

!!! warning "Subscription lifecycle"
    Reconnect flows can invalidate active subscriptions. Re-apply subscriptions after connection restoration.

## Related Docs

- [Events API](events.md)
- [Chatstate Namespace](chatstate.md)
- [Security Practices](../operations/security.md)
