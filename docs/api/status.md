# Status Namespace (`client.status`)

Use `client.status` to publish ephemeral status content and control audience privacy.

## Core Types

- `StatusPrivacySetting`: `Contacts`, `AllowList`, `DenyList`
- `StatusSendOptions(privacy=...)`

## Method Matrix

| Method | Purpose |
| --- | --- |
| `send_text(text, background_argb, font, recipients, options=None)` | Publish text status |
| `send_image(upload, thumbnail, recipients, caption=None, options=None)` | Publish image status |
| `send_video(upload, thumbnail, duration_seconds, recipients, caption=None, options=None)` | Publish video status |
| `send_raw(message, recipients, options=None)` | Publish custom raw message payload |
| `revoke(message_id, recipients, options=None)` | Revoke previously published status |
| `default_privacy()` | Default privacy helper |

## Runnable Example: Text Status Broadcast

```python
from tryx.client import StatusSendOptions, StatusPrivacySetting


async def publish_status(client, recipients, text):
    options = StatusSendOptions(privacy=StatusPrivacySetting.Contacts)
    message_id = await client.status.send_text(
        text=text,
        background_argb=0xFF1F9D86,
        font=1,
        recipients=recipients,
        options=options,
    )
    return message_id
```

## Runnable Example: Media Status

```python
from tryx.wacore import MediaType


async def publish_image_status(client, recipients, image_path, thumbnail_bytes):
    upload = await client.upload_file(image_path, MediaType.Image)
    return await client.status.send_image(
        upload=upload,
        thumbnail=thumbnail_bytes,
        recipients=recipients,
        caption="Weekly update",
    )
```

!!! tip "Privacy-first workflow"
    Fetch and enforce your privacy model before status publication, especially in mixed audiences.

## Related Docs

- [Privacy Namespace](privacy.md)
- [Helpers API](helpers.md)
- [Tutorial: Media Workflows](../tutorials/media-workflows.md)
