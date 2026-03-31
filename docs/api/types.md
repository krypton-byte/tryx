# Types API

Core value objects in `tryx.types` define identity, message metadata, and result contracts used across client namespaces.

## Identity and Source Types

### `JID`

Canonical WhatsApp identifier:

- `user`: account or group numeric/opaque id
- `server`: domain segment (`s.whatsapp.net`, `g.us`, etc.)

### `MessageSource`

Routing context for a message:

- sender identity
- target chat identity
- from-me and group flags
- alternate identity hints for multi-device routing

### `MessageInfo`

Message metadata envelope:

- `id`
- `type`
- `timestamp`
- `source`
- edit and verification metadata

## Send and Media Result Types

| Type | Produced by |
| --- | --- |
| `UploadResponse` | `upload`, `upload_file` |
| `SendResult` | `send_*` methods |
| `MediaReuploadResult` | `request_media_reupload` |
| `ProfilePicture` | `client.contact.get_profile_picture` |

## Advanced Metadata Types

- `MsgBotInfo`
- `MsgMetaInfo`
- `DeviceSentMeta`

These are useful when building sync-aware systems and diagnostics.

## Practical Typed Patterns

=== "Event extraction"
    ```python
    from tryx.events import EvMessage
    from tryx.types import JID


    def source_chat(event: EvMessage) -> JID:
        return event.data.message_info.source.chat
    ```

=== "Result-safe send"
    ```python
    async def send_with_audit(client, chat, text):
        result = await client.send_text(chat, text)
        return {"message_id": result.id, "ts": result.timestamp}
    ```

## Privacy Type Bridge

Privacy and status workflows (documented in [Privacy Namespace](privacy.md) and [Status Namespace](status.md)) rely on typed enums from the client surface, including:

- `PrivacyCategory`
- `PrivacyValue`
- `DisallowedListAction`
- `StatusPrivacySetting`

!!! tip "Type-first architecture"
    Keep handler boundaries typed. Convert external payloads into typed objects early, and keep business logic free of ad-hoc dict parsing.
