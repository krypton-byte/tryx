# Profile Namespace (`client.profile`)

`client.profile` controls account-facing profile presentation.

## Methods

| Method | Purpose |
| --- | --- |
| `set_push_name(name)` | Update display/push name |
| `set_status_text(text)` | Update profile status/about text |
| `set_profile_picture(image_data)` | Upload and set profile picture |
| `remove_profile_picture()` | Remove profile picture |

## Runnable Example: Admin Profile Commands

```python
from tryx.events import EvMessage


@app.on(EvMessage)
async def profile_admin(client, event):
    text = (event.data.get_text() or "").strip()
    chat = event.data.message_info.source.chat

    if text.startswith("/profile name "):
        value = text.split(maxsplit=2)[2]
        await client.profile.set_push_name(value)
        await client.send_text(chat, "Push name updated")

    elif text.startswith("/profile status "):
        value = text.split(maxsplit=2)[2]
        await client.profile.set_status_text(value)
        await client.send_text(chat, "Status text updated")
```

## Runnable Example: Picture Rotation

```python
async def rotate_profile_picture(client, image_bytes):
    pic_id = await client.profile.set_profile_picture(image_bytes)
    return {"picture_id": pic_id}
```

## Operational Notes

=== "Lifecycle"
    1. Read desired profile state from config.
    2. Apply updates during startup.
    3. Use explicit admin commands for runtime changes.

=== "Safety"
    - Keep raw image bytes validated and size-limited before upload.
    - Log profile mutations with operator identity.

!!! tip "Consistency"
    Couple profile changes with privacy policy reviews if your client identity is user-facing.

## Related Docs

- [Privacy Namespace](privacy.md)
- [Security Practices](../operations/security.md)
- [Tutorial: Profile and Privacy](../tutorials/profile-privacy.md)
