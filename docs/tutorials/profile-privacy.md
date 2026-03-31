# Tutorial: Profile and Privacy

Build admin-controlled profile and privacy management commands.

!!! note "Use case"
    This pattern is useful for multi-operator bots that need strict control over account identity and visibility rules.

## Step 1: Profile Commands

```python
from tryx.events import EvMessage

ADMIN = "1234567890"


@bot.on(EvMessage)
async def profile_commands(client, event):
    sender = event.data.message_info.source.sender.user
    if sender != ADMIN:
        return

    text = (event.data.get_text() or "").strip()
    chat = event.data.message_info.source.chat

    if text.startswith("/profile name "):
        await client.profile.set_push_name(text.split(maxsplit=2)[2])
        await client.send_text(chat, "Name updated")

    elif text.startswith("/profile status "):
        await client.profile.set_status_text(text.split(maxsplit=2)[2])
        await client.send_text(chat, "Status updated")
```

## Step 2: Privacy Category Commands

```python
from tryx.client import PrivacyCategory, PrivacyValue


@bot.on(EvMessage)
async def privacy_commands(client, event):
    text = (event.data.get_text() or "").strip()
    chat = event.data.message_info.source.chat

    if text == "/privacy list":
        rows = await client.privacy.fetch_settings()
        lines = [f"{row.category}: {row.value}" for row in rows]
        await client.send_text(chat, "\n".join(lines))

    elif text == "/privacy status contacts":
        await client.privacy.set_setting(PrivacyCategory.Status, PrivacyValue.Contacts)
        await client.send_text(chat, "Status visibility set to contacts")
```

## Step 3: Disallowed List Example

```python
from tryx.client import DisallowedListAction, DisallowedListUpdate, DisallowedListUserEntry


async def block_visibility_for(client, target_jid):
    update = DisallowedListUpdate(
        dhash="",
        users=[DisallowedListUserEntry(action=DisallowedListAction.Add, jid=target_jid)],
    )
    await client.privacy.set_disallowed_list(PrivacyCategory.Status, update)
```

## Operational Guidance

=== "Governance"
    - Keep profile/privacy commands admin-only.
    - Log every policy change.

=== "Safety"
    - Validate category/value command input before applying.
    - Rate-limit mutation commands.

## Related Docs

- [Privacy Namespace](../api/privacy.md)
- [Profile Namespace](../api/profile.md)
- [Security Practices](../operations/security.md)
