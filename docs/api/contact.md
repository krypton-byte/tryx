# Contact Namespace (`client.contact`)

Use this namespace for user discovery, registration checks, and profile picture metadata.

!!! tip "When to use this namespace"
    Use `client.contact` before sending high-value outbound messages so you can validate registration and enrich context.

## Method Matrix

| Method | Input | Output | Notes |
| --- | --- | --- | --- |
| `get_info(phones)` | `list[str]` (phone numbers) | `list[ContactInfo]` | Batch phone lookup |
| `get_user_info(jid)` | `JID` | `dict[JID, UserInfo]` | Rich user profile map |
| `get_profile_picture(jid, preview)` | `JID`, `bool` | `ProfilePicture` | Preview/full modes |
| `is_on_whatsapp(jids)` | `list[JID]` | `list[IsOnWhatsAppResult]` | Registration verification |

## Technical Notes

=== "Input Rules"
    - Keep phone values normalized (E.164-style digits without separators where possible).
    - Use `JID(user="<phone>", server="s.whatsapp.net")` for direct identity checks.
    - Batch operations are better than one-by-one calls for throughput.

=== "Output Semantics"
    - `ContactInfo` may include `status` and business flags.
    - `UserInfo` returns richer metadata and can include `lid` mappings.
    - `ProfilePicture` may contain URLs and identifier metadata; treat missing fields as valid state.

## Runnable Example: Registration Gate

```python
from tryx.events import EvMessage
from tryx.types import JID


@bot.on(EvMessage)
async def on_lookup(client, event):
    text = (event.data.get_text() or "").strip()
    if not text.startswith("/check "):
        return

    phone = text.split(maxsplit=1)[1]
    target = JID(user=phone, server="s.whatsapp.net")

    result = await client.contact.is_on_whatsapp([target])
    row = result[0]

    chat = event.data.message_info.source.chat
    if row.is_registered:
        await client.send_text(chat, f"{phone} is registered")
    else:
        await client.send_text(chat, f"{phone} is not registered")
```

## Advanced Example: Enrichment Before Outreach

```python
async def enrich_contacts(client, phones: list[str]) -> dict[str, str]:
    rows = await client.contact.get_info(phones)
    out = {}
    for row in rows:
        kind = "business" if row.is_business else "personal"
        out[row.jid.user] = f"{kind} | status={row.status or '-'}"
    return out
```

!!! warning "Common Pitfalls"
    - Do not assume every lookup returns a profile picture.
    - Avoid repeated single-item calls in loops; use batch methods.
    - Treat optional fields (`status`, `picture_id`, `lid`) as nullable.

## Related Docs

- [Types API](types.md)
- [Profile Namespace](profile.md)
- [Tutorial: Command Bot](../tutorials/command-bot.md)
