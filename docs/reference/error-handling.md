# Error Handling

Use exception classes and failure classification to decide whether to retry,
fail fast, or trigger operator action.

---

## Exception Classes

| Exception | Category | Retryable | Description |
|-----------|----------|-----------|-------------|
| `FailedBuildBot` | Config | No | Bot initialization failed |
| `FailedToDecodeProto` | Payload | No | Protobuf deserialization failed |
| `EventDispatchError` | Runtime | Sometimes | Handler dispatch layer issue |
| `PyPayloadBuildError` | Payload | No | Outbound message construction failed |
| `UnsupportedBackend` | Config | No | Backend type not recognized |
| `UnsupportedEventType` | Config | No | Event type not registered |

**Backward-compatible aliases:**

| Alias | Maps To |
|-------|---------|
| `BuildBotError` | `FailedBuildBot` |
| `UnsupportedBackendError` | `UnsupportedBackend` |
| `UnsupportedEventTypeError` | `UnsupportedEventType` |

---

## Failure Classification

| Category | Typical Exceptions | Retry? | Fix |
|----------|-------------------|--------|-----|
| **Payload/shape** | `PyPayloadBuildError`, `FailedToDecodeProto` | No | Fix input data |
| **Dispatch/runtime** | `EventDispatchError` | Sometimes | Check handler code |
| **Configuration** | `UnsupportedBackend`, `UnsupportedEventType` | No | Fix config or code |

---

## Strategy

1. **Catch specific Tryx exception classes first** — don't fall through to bare `Exception`
2. **Add contextual logs** — include `chat_jid`, `message_id`, handler name
3. **Use retry only for transient failures** — network, temporary server errors
4. **Avoid retry loops on structural errors** — payload shape, config issues

---

## Pattern: Basic Error Handling

```python
try:
    await client.send_text(chat_jid, "hello")
except PyPayloadBuildError as exc:
    # Non-retryable: payload construction issue
    logger.error(f"Payload error: {exc}")
except EventDispatchError:
    # Potentially retryable: dispatch layer issue
    logger.warning("Dispatch error, retrying...")
    await retry(lambda: client.send_text(chat_jid, "hello"), attempts=3)
```

---

## Pattern: Namespace-Aware Handling

### Messaging and Chat Actions

Validate target JID and payload before retry:

```python
try:
    await client.send_text(chat_jid, "hello")
except PyPayloadBuildError:
    # JID or payload invalid — don't retry
    await client.send_text(chat_jid, "Failed: invalid payload")
except EventDispatchError:
    # Retry transient failure
    await retry(lambda: client.send_text(chat_jid, "hello"))
```

### Group/Community Mutations

Inspect per-participant response details:

```python
try:
    results = await client.groups.add_participants(group_jid, participants)
    for r in results:
        if r.error_code:
            logger.warning(f"Participant {r.jid}: {r.error_code}")
except Exception as e:
    logger.error(f"Group mutation failed: {e}")
```

### Poll and Media Flows

Persist context before recovery attempts:

```python
# Store poll metadata first
poll_id, secret = await client.polls.create(to=chat_jid, name="Q", options=["A", "B"], selectable_count=1)

# Now if vote fails, we have the poll_id and secret for recovery
try:
    await client.polls.vote(chat_jid, poll_id, creator_jid, secret, ["A"])
except Exception as e:
    logger.error(f"Vote failed for poll {poll_id}: {e}")
    # We can retry using stored poll_id and secret
```

---

## Pattern: Structured Logging

```python
import logging

logger = logging.getLogger("bot")


@app.on(EvMessage)
async def safe_handler(client, event):
    message_id = event.data.message_info.id
    chat = event.data.message_info.source.chat

    try:
        text = event.data.get_text() or ""
        await client.send_text(chat, f"Echo: {text}")
    except PyPayloadBuildError as e:
        logger.error(
            "payload_error",
            extra={
                "message_id": message_id,
                "chat": str(chat),
                "error": str(e),
            },
        )
    except Exception as e:
        logger.error(
            "handler_error",
            extra={
                "message_id": message_id,
                "chat": str(chat),
                "error": str(e),
                "type": type(e).__name__,
            },
            exc_info=True,
        )
```

---

## Handler-Level Protection

Wrap your entire event handler to prevent one bad event from crashing
your bot:

```python
@app.on(EvMessage)
async def protected_handler(client, event):
    try:
        await process_message(client, event)
    except Exception as e:
        logger.error(f"Handler failed: {e}", exc_info=True)
        # Don't crash — Tryx continues dispatching
```

!!! warning "Handler exceptions"
    If a handler raises an exception, Tryx catches it and continues
    dispatching other events. However, the error is logged and may affect
    your bot's reliability. Always handle errors explicitly.

---

## Incident Response

Always log these fields for post-mortem analysis:

| Field | Example |
|-------|---------|
| `message_id` | `3EB0A1B2C3D4E5F6` |
| `chat_jid` | `5599800001@s.whatsapp.net` |
| `handler_name` | `on_message` |
| `exception_type` | `PyPayloadBuildError` |
| `timestamp` | `2025-01-15T10:30:00Z` |

---

## Related

- [Reliability Playbook](../operations/reliability.md) — production error handling
- [Troubleshooting](../operations/troubleshooting.md) — connection and event issues
- [QnA](../faq/qna.md) — common questions
