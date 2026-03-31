# Types API

Core data classes live in `tryx.types`.

## JID

Represents a WhatsApp identifier with `user` and `server` parts.

## MessageSource

Describes where a message came from and how it was addressed.

Fields include:

- sender
- chat
- is_from_me
- is_group
- alternate/recipient addressing metadata

## MessageInfo

Common metadata for inbound/outbound messages:

- `id`
- `type`
- `timestamp`
- `source`
- edit metadata and verification hints

## Bot/Meta Side Types

- `MsgBotInfo`
- `MsgMetaInfo`
- `DeviceSentMeta`

These provide advanced metadata used in sync/edit contexts.

## Media and Send Result Types

- `UploadResponse`
- `SendResult`
- `MediaReuploadResult`

## Contact Visual Type

- `ProfilePicture`

## Practical Pattern

Use typed helper functions around event payloads to keep business logic clean:

```python
from tryx.events import EvMessage
from tryx.types import JID


def source_chat(event: EvMessage) -> JID:
    return event.data.message_info.source.chat
```
