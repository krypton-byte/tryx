# Event Model

Tryx emits typed event classes from `tryx.events`. Every event has a known payload shape.

## Handler Registration

```python
@bot.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    ...
```

## Event Categories

- Lifecycle: `EvConnected`, `EvDisconnected`, `EvLoggedOut`
- Pairing: `EvPairingQrCode`, `EvPairingCode`, `EvPairSuccess`, `EvPairError`
- Messaging: `EvMessage`, `EvReceipt`, `EvUndecryptableMessage`
- Chat actions sync: archive, mute, mark-read, delete-chat, delete-for-me
- Presence and profile: chat presence, availability, picture, push-name, about
- Contact and device sync: contact update, device list update
- Group and newsletter updates

## Event Payload Pattern

Many events expose a lazy `data` property:

- `event.data` returns a rich typed object
- conversion from Rust internals happens on demand
- repeated access often reuses cached object instances

## Important Reliability Notes

- Callback execution order is event-driven; do not assume strict timing between different event classes.
- Keep handlers short and non-blocking.
- For expensive work, queue to background tasks.

## Best Practices

1. Validate optional fields before use.
2. Prefer exact event classes over broad dynamic checks.
3. Log enough metadata (`jid`, `message_id`, timestamps) for debugging.
4. Treat undecryptable and sync events as normal runtime states, not always errors.
