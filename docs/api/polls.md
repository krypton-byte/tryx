# Polls Namespace (`client.polls`)

`client.polls` supports encrypted poll workflows: create, vote, decrypt, and aggregate.

!!! note "Crypto lifecycle"
    `create(...)` returns `(poll_msg_id, message_secret)`. Persist both values if you plan to decrypt and aggregate votes later.

## Method Matrix

| Method | Purpose |
| --- | --- |
| `create(to, name, options, selectable_count)` | Create poll and return secret |
| `vote(chat_jid, poll_msg_id, poll_creator_jid, message_secret, option_names)` | Submit encrypted vote |
| `decrypt_vote(...)` | Decrypt one encrypted vote payload |
| `aggregate_votes(...)` | Compute per-option tally from encrypted vote rows |

## Runnable Example: Create + Vote

```python
from tryx.types import JID


async def create_poll(client, chat: JID):
    poll_id, secret = await client.polls.create(
        to=chat,
        name="Deploy window?",
        options=["Tonight", "Tomorrow", "Next week"],
        selectable_count=1,
    )
    return poll_id, secret


async def cast_vote(client, chat, poll_id, creator_jid, secret):
    return await client.polls.vote(
        chat_jid=chat,
        poll_msg_id=poll_id,
        poll_creator_jid=creator_jid,
        message_secret=secret,
        option_names=["Tomorrow"],
    )
```

## Runnable Example: Aggregate Encrypted Votes

```python
from tryx.types import JID


async def tally(client, poll_options, encrypted_rows, secret, poll_id, creator_jid: JID):
    # encrypted_rows: list[tuple[JID, bytes, bytes]]
    return client.polls.aggregate_votes(
        poll_options=poll_options,
        votes=encrypted_rows,
        message_secret=secret,
        poll_msg_id=poll_id,
        poll_creator_jid=creator_jid,
    )
```

## Pitfalls and Controls

!!! warning "Secret management"
    If `message_secret` is lost, vote decryption and tallying are no longer possible for that poll.

!!! tip "Storage strategy"
    Store `(poll_msg_id, message_secret)` in durable backend storage keyed by chat and poll creator identity.

## Related Docs

- [Helpers API](helpers.md)
- [Types API](types.md)
- [Tutorial: Poll Survey Workflow](../tutorials/poll-survey.md)
