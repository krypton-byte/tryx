# Chat Actions Namespace (`client.chat_actions`)

`client.chat_actions` manages chat state transitions and message-level actions (edit, revoke, react, star, archive, mute).

!!! note "Why this namespace matters"
    Many WhatsApp state changes are synchronization events. If your client writes chat state, subscribe to related events so your local view stays consistent.

## Builder Helpers

### `build_message_key(id, remote_jid, from_me, participant=None)`
Build a canonical `MessageKey` used by advanced sync actions.

### `build_message_range(last_message_timestamp, last_system_message_timestamp, messages)`
Build `SyncActionMessageRange` for operations that need explicit message windows.

## Method Groups

=== "Chat Level"
    - `archive_chat(jid, message_range=None)`
    - `unarchive_chat(jid, message_range=None)`
    - `pin_chat(jid)`
    - `unpin_chat(jid)`
    - `mute_chat(jid)`
    - `mute_chat_until(jid, mute_end_timestamp_ms)`
    - `unmute_chat(jid)`
    - `mark_chat_as_read(jid, read, message_range=None)`
    - `delete_chat(jid, delete_media, message_range=None)`

=== "Message Level"
    - `star_message(chat_jid, participant_jid, message_id, from_me)`
    - `unstar_message(chat_jid, participant_jid, message_id, from_me)`
    - `delete_message_for_me(chat_jid, participant_jid, message_id, from_me, delete_media, message_timestamp=None)`
    - `edit_message(chat_jid, original_id, new_message)`
    - `revoke_message(chat_jid, message_id, original_sender=None)`
    - `react_message(chat_jid, message_id, reaction, from_me=False, participant_jid=None)`

## Runnable Example: Moderation Utility

```python
from tryx.events import EvMessage


@app.on(EvMessage)
async def moderation_actions(client, event):
    text = (event.data.get_text() or "").strip()
    chat = event.data.message_info.source.chat

    if text == "/pin":
        await client.chat_actions.pin_chat(chat)
        await client.send_text(chat, "Chat pinned")
    elif text == "/unpin":
        await client.chat_actions.unpin_chat(chat)
        await client.send_text(chat, "Chat unpinned")
```

## Runnable Example: React + Revoke Flow

```python
async def resolve_ticket(client, chat_jid, msg_id):
    await client.chat_actions.react_message(chat_jid, msg_id, "✅")
    # Optional follow-up: revoke a stale client message
    await client.chat_actions.revoke_message(chat_jid, msg_id)
```

## Operational Guidance

!!! tip "Idempotent wrappers"
    Wrap mutation methods in idempotent helpers in case your handler retries after transient errors.

!!! warning "Message identity"
    For group messages, include participant identity when needed. Wrong `(message_id, participant)` combinations may lead to no-op operations.

## Related Docs

- [Events API](events.md)
- [Types API](types.md)
- [Groups Namespace](groups.md)
