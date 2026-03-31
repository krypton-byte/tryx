# Newsletter Namespace (`client.newsletter`)

`client.newsletter` manages newsletter/channel discovery, join/leave, messaging, reactions, and message history.

## Method Matrix

| Method | Purpose |
| --- | --- |
| `list_subscribed()` | List subscriptions |
| `get_metadata(jid)` | Metadata by newsletter JID |
| `get_metadata_by_invite(invite_code)` | Metadata via invite code |
| `create(name, description=None)` | Create a newsletter |
| `join(jid)` / `leave(jid)` | Membership lifecycle |
| `update(jid, name=None, description=None)` | Edit metadata |
| `subscribe_live_updates(jid)` | Enable live updates stream |
| `send_message(jid, message)` | Publish message |
| `send_reaction(jid, server_id, reaction)` | React to a message |
| `get_messages(jid, count, before=None)` | Fetch history window |

## Runnable Example: Subscribe and Post

```python
from tryx.helpers import NewsletterHelpers


async def post_update(client, invite_code: str, text: str):
    metadata = await client.newsletter.get_metadata_by_invite(invite_code)
    await client.newsletter.join(metadata.jid)

    message = NewsletterHelpers.build_text_message(text)
    server_message_id = await client.newsletter.send_message(metadata.jid, message)
    return metadata.name, server_message_id
```

## Runnable Example: History Query Window

```python
async def pull_recent(client, newsletter_jid, count=20):
    rows = await client.newsletter.get_messages(newsletter_jid, count)
    return [
        {
            "server_id": row.server_id,
            "type": row.message_type,
            "timestamp": row.timestamp,
        }
        for row in rows
    ]
```

## Technical Guidance

=== "Publishing"
    - Build protobuf messages using [Helpers API](helpers.md) when possible.
    - Keep content idempotent if handlers might retry.

=== "History"
    - Use `before` cursor for backfill pagination.
    - Treat message type variants as dynamic; not every row is plain text.

!!! tip "Live updates"
    Pair `subscribe_live_updates` with event handlers so your process can react to new newsletter activity without polling loops.

## Related Docs

- [Helpers API](helpers.md)
- [Events API](events.md)
- [Polls Namespace](polls.md)
