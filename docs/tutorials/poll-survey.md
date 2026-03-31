# Tutorial: Poll Survey Workflow

Create poll surveys, collect encrypted votes, and aggregate results.

## Step 1: Create Poll

```python
poll_id, secret = await client.polls.create(
    to=chat_jid,
    name="Deploy this week?",
    options=["Yes", "No"],
    selectable_count=1,
)
```

Store `poll_id` and `secret` in persistent storage.

!!! warning
    Without `secret`, encrypted vote payloads cannot be decrypted later.

## Step 2: Cast Vote

```python
await client.polls.vote(
    chat_jid=chat_jid,
    poll_msg_id=poll_id,
    poll_creator_jid=creator_jid,
    message_secret=secret,
    option_names=["Yes"],
)
```

## Step 3: Aggregate Vote Rows

```python
# encrypted_rows: list[tuple[JID, bytes, bytes]]
results = client.polls.aggregate_votes(
    poll_options=["Yes", "No"],
    votes=encrypted_rows,
    message_secret=secret,
    poll_msg_id=poll_id,
    poll_creator_jid=creator_jid,
)
```

## Display Results

```python
lines = [f"{row.name}: {len(row.voters)}" for row in results]
await client.send_text(chat_jid, "\n".join(lines))
```

## Production Checklist

- Persist poll metadata (`poll_id`, `secret`, creator identity).
- Validate option names before vote submission.
- Keep vote aggregation idempotent.
- Record processing checkpoint to avoid duplicate tally updates.

## Related Docs

- [Polls Namespace](../api/polls.md)
- [Helpers API](../api/helpers.md)
- [Reliability](../operations/reliability.md)
