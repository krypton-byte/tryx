# :material-shield-check: Reliability Playbook

This page focuses on idempotency, retry strategy, and safe handler design.

## Reliability Pillars

| Pillar | Why |
| --- | --- |
| Idempotency | Prevent duplicate side effects |
| Bounded retries | Recover transient failures safely |
| Queue delegation | Keep event handlers responsive |
| Structured logging | Make incident triage fast |

## Idempotency Pattern

```python
processed: set[str] = set()


@app.on(EvMessage)
async def reliable_handler(client, event):
    msg_id = event.data.message_info.id
    if msg_id in processed:
        return
    processed.add(msg_id)

    await client.send_text(event.data.message_info.source.chat, "processed")
```

## Retry Envelope

```python
async def retry(coro_factory, attempts=3):
    last_exc = None
    for _ in range(attempts):
        try:
            return await coro_factory()
        except Exception as exc:
            last_exc = exc
    raise last_exc
```

!!! tip "Do not retry everything"
    Structural payload errors should fail fast. Reserve retries for network/transient failures.

---

## Retry with Exponential Backoff

When calling WhatsApp APIs (sending messages, uploading media), transient
failures are common. Use exponential backoff to avoid hitting rate limits
and to give servers time to recover.

### Basic Exponential Backoff

```python
import asyncio
import random


async def retry_with_backoff(
    coro_factory,
    max_attempts: int = 5,
    base_delay: float = 1.0,
    max_delay: float = 60.0,
    jitter: bool = True,
):
    """Retry a coroutine factory with exponential backoff.

    Args:
        coro_factory: Callable that returns a new coroutine each call.
        max_attempts: Maximum number of attempts (including the first).
        base_delay: Initial delay in seconds between retries.
        max_delay: Maximum delay cap in seconds.
        jitter: Add random jitter to prevent thundering herd.

    Returns:
        The result of the coroutine.

    Raises:
        The last exception if all attempts fail.
    """
    last_exc = None
    for attempt in range(max_attempts):
        try:
            return await coro_factory()
        except Exception as exc:
            last_exc = exc
            if attempt == max_attempts - 1:
                break
            delay = min(base_delay * (2**attempt), max_delay)
            if jitter:
                delay *= random.uniform(0.5, 1.0)
            await asyncio.sleep(delay)
    raise last_exc
```

### Usage Example

```python
@app.on(EvMessage)
async def send_with_retry(client, event):
    chat = event.data.message_info.source.chat
    text = event.data.get_text() or ""

    if text.lower() == "ping":
        await retry_with_backoff(
            lambda: client.send_text(chat, "pong"),
            max_attempts=3,
            base_delay=1.0,
        )
```

### When to Use Backoff

| Scenario | Strategy |
| --- | --- |
| Send message fails | Retry 2-3 times with 1-2s base delay |
| Upload media fails | Retry 3-5 times with 2s base delay |
| Network timeout | Retry 3 times with exponential backoff |
| `RuntimeError` from client | Do NOT retry — likely a session/auth issue |
| `EvTemporaryBan` received | Pause all operations, resume after ban expires |

### When NOT to Retry

- **Authentication errors** — Re-pair instead of retrying
- **Invalid payload** — Fix the message, don't resend as-is
- **`EvLoggedOut` / `EvStreamReplaced`** — Session is gone, re-initialize
- **Permanent protocol errors** — Check message construction logic

---

## Rate Limit Awareness

WhatsApp enforces rate limits. Exceeding them causes temporary bans
(visible via `EvTemporaryBan`). To avoid this:

- **Batch messages** — Send to multiple recipients with delays between
  sends, not in a tight loop.
- **Respect ban expiry** — When `EvTemporaryBan` fires, stop sending
  entirely until the ban expires.
- **Monitor message volume** — Track how many messages per minute your
  bot sends and stay well below WhatsApp's limits.
- **Spread work over time** — Use `asyncio.sleep()` between bulk
  operations.

```python
@app.on(EvMessage)
async def rate_limited_sender(client, event):
    chat = event.data.message_info.source.chat
    text = event.data.get_text() or ""

    if text == "!broadcast":
        recipients = [jid1, jid2, jid3]
        for recipient in recipients:
            await client.send_text(recipient, "Hello everyone!")
            await asyncio.sleep(1.0)  # 1 second between sends
```

### Temporary Ban Handling

```python
@app.on(EvTemporaryBan)
async def on_ban(client, event):
    ban_data = event.data
    print(f"Banned: {ban_data.code}, expires: {ban_data.expire}")
    # Stop all sending operations until ban expires
    # Log the event for monitoring
```

---

## Recommended Patterns Summary

| Pattern | When | How |
| --- | --- | --- |
| Simple retry | Transient network failure | Fixed delay, 3 attempts |
| Exponential backoff | API rate limits, server errors | Delay doubles each attempt, capped |
| No retry | Auth errors, invalid payload | Fail fast, fix root cause |
| Rate limiting | Bulk sending, broadcast | `asyncio.sleep()` between sends |
| Ban detection | High-frequency operations | Listen for `EvTemporaryBan` |

## Handler Throughput Pattern

- Parse and validate quickly in event handler.
- Enqueue heavy work to background worker.
- Acknowledge user quickly with minimal response.

## Related Docs

- [Command Automation Tutorial](../tutorials/command-bot.md)
- [Performance Guide](performance.md)
- [Error Handling](../reference/error-handling.md)
