# Blocking Namespace (`client.blocking`)

Manage blocklist lifecycle and block-state checks.

## Methods

| Method | Output | Notes |
| --- | --- | --- |
| `block(jid)` | `None` | Block one user |
| `unblock(jid)` | `None` | Remove block |
| `get_blocklist()` | `list[BlocklistEntry]` | Includes timestamp metadata |
| `is_blocked(jid)` | `bool` | Fast single-user check |

## Runnable Example: Auto-Quarantine

```python
spam_hits: dict[str, int] = {}
LIMIT = 5


@bot.on(EvMessage)
async def anti_spam(client, event):
    sender = event.data.message_info.source.sender
    key = sender.user

    spam_hits[key] = spam_hits.get(key, 0) + 1
    if spam_hits[key] < LIMIT:
        return

    await client.blocking.block(sender)
    await client.send_text(event.data.message_info.source.chat, "User has been blocked")
    spam_hits.pop(key, None)
```

## Runnable Example: Time-boxed Blocklist Cleanup

```python
from datetime import datetime, timedelta


async def cleanup_blocklist(client):
    blocklist = await client.blocking.get_blocklist()
    cutoff = datetime.utcnow() - timedelta(days=30)

    for row in blocklist:
        if row.timestamp is None:
            continue
        when = datetime.utcfromtimestamp(row.timestamp / 1000)
        if when < cutoff:
            await client.blocking.unblock(row.jid)
```

!!! tip "Identity checks"
    Pair `is_blocked` with `BlockingHelpers.same_user(...)` if you work with multiple server variants of the same user identity.

## Related Docs

- [Helpers API](helpers.md)
- [Privacy Namespace](privacy.md)
- [Security Practices](../operations/security.md)
