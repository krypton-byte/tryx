# Error Handling

Use exception classes and failure classification to decide whether to retry, fail fast, or trigger operator action.

## Exception Classes

- `FailedBuildBot`
- `FailedToDecodeProto`
- `EventDispatchError`
- `PyPayloadBuildError`
- `UnsupportedBackend`
- `UnsupportedEventType`

Backward-compatible aliases:

- `BuildBotError`
- `UnsupportedBackendError`
- `UnsupportedEventTypeError`

## Failure Classification

| Category | Typical exceptions | Retry? |
| --- | --- | --- |
| Payload/shape issues | `PyPayloadBuildError`, `FailedToDecodeProto` | No (fix input) |
| Dispatch/runtime issues | `EventDispatchError` | Sometimes |
| Configuration issues | `UnsupportedBackend`, `UnsupportedEventType` | No (fix config/code) |

## Strategy

1. Catch specific Tryx exception classes first.
2. Add contextual logs (`chat_jid`, `message_id`, handler name).
3. Use retry only for transient failures.
4. Avoid retry loops on structural payload errors.

## Namespace-aware Guidance

- Messaging and chat actions: validate target JID and payload before retry.
- Group/community mutations: inspect per-participant response details.
- Poll and media flows: persist context (`poll_id`, secrets, media metadata) before recovery attempts.

## Pattern Example

```python
try:
    await client.send_text(chat, "ok")
except PyPayloadBuildError as exc:
    # payload construction issue
    raise
except EventDispatchError:
    # callback dispatch layer issue
    raise
```

## Pattern Example With Classification

```python
try:
    await client.send_text(chat_jid, "ok")
except PyPayloadBuildError:
    # non-retryable
    await client.send_text(chat_jid, "payload invalid")
except EventDispatchError:
    # retry envelope can be applied
    await retry(lambda: client.send_text(chat_jid, "ok"), attempts=3)
```

!!! tip "Incident response"
    Always log `message_id`, `chat_jid`, handler name, and exception type for post-mortem analysis.
