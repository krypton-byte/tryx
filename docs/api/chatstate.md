# Chatstate Namespace (`client.chatstate`)

Use chatstate signals to indicate user-facing activity such as typing or recording.

## Methods

| Method | Purpose |
| --- | --- |
| `send(to, state)` | Send explicit `ChatStateType` |
| `send_composing(to)` | Typing indicator |
| `send_recording(to)` | Recording indicator |
| `send_paused(to)` | Pause/stop indicator |

## State Values

- `ChatStateType.Composing`
- `ChatStateType.Recording`
- `ChatStateType.Paused`

## Runnable Example: Latency-Hiding Reply

```python
@app.on(EvMessage)
async def smart_reply(client, event):
    chat = event.data.message_info.source.chat

    await client.chatstate.send_composing(chat)
    try:
        answer = await expensive_lookup(event.data.get_text() or "")
    finally:
        await client.chatstate.send_paused(chat)

    await client.send_text(chat, answer, quoted=event)
```

## Runnable Example: Voice Pipeline

```python
async def send_voice(client, chat, audio_bytes):
    await client.chatstate.send_recording(chat)
    await client.send_audio(chat, audio_bytes, ptt=True)
    await client.chatstate.send_paused(chat)
```

!!! warning "Signal hygiene"
    If your handler crashes after sending composing/recording, the UX signal may look stale.
    Wrap long operations in `try/finally` and always send paused.

## Related Docs

- [Presence Namespace](presence.md)
- [Events API](events.md)
- [Tutorial: Command Automation](../tutorials/command-bot.md)
