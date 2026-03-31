# Error Handling

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

## Strategy

1. Catch specific Tryx exception classes first.
2. Add contextual logs (`chat_jid`, `message_id`, handler name).
3. Use retry only for transient failures.
4. Avoid retry loops on structural payload errors.

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
