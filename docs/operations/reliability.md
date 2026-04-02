# Reliability Playbook

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

## Handler Throughput Pattern

- Parse and validate quickly in event handler.
- Enqueue heavy work to background worker.
- Acknowledge user quickly with minimal response.

## Related Docs

- [Command Automation Tutorial](../tutorials/command-bot.md)
- [Performance Guide](performance.md)
- [Error Handling](../reference/error-handling.md)
