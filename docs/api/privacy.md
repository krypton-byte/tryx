# Privacy Namespace (`client.privacy`)

Use `client.privacy` to read and mutate account privacy categories, disallowed lists, and default disappearing mode.

## Methods

| Method | Purpose |
| --- | --- |
| `fetch_settings()` | Retrieve privacy category/value pairs |
| `set_setting(category, value)` | Set one category privacy mode |
| `set_disallowed_list(category, update)` | Add/remove disallowed users |
| `set_default_disappearing_mode(duration_seconds)` | Set default disappearing timer |

## Core Enums

### PrivacyCategory

`Last`, `Online`, `Profile`, `Status`, `GroupAdd`, `ReadReceipts`, `CallAdd`, `Messages`, `DefenseMode`, `Other`

### PrivacyValue

`All`, `Contacts`, `None_`, `ContactBlacklist`, `MatchLastSeen`, `Known`, `Off`, `OnStandard`, `Other`

### DisallowedListAction

`Add`, `Remove`

## Runnable Example: Fetch and Render

```python
async def dump_privacy(client):
    rows = await client.privacy.fetch_settings()
    return {str(row.category): str(row.value) for row in rows}
```

## Runnable Example: Category + Disallowed Update

```python
from tryx.client import DisallowedListAction, DisallowedListUpdate, DisallowedListUserEntry


async def hide_status_from(client, target_jid):
    await client.privacy.set_setting(category=PrivacyCategory.Status, value=PrivacyValue.ContactBlacklist)

    update = DisallowedListUpdate(
        dhash="",
        users=[
            DisallowedListUserEntry(
                action=DisallowedListAction.Add,
                jid=target_jid,
            )
        ],
    )
    await client.privacy.set_disallowed_list(PrivacyCategory.Status, update)
```

## Runnable Example: Default Disappearing Mode

```python
# 0 = off, 86400 = 1 day, 604800 = 1 week, 7776000 = 90 days
await client.privacy.set_default_disappearing_mode(604800)
```

!!! tip "Race-safe updates"
    Keep track of list versioning strategy (`dhash`) in your own domain logic when doing frequent disallowed-list writes.

!!! warning "Do not hardcode assumptions"
    Category/value compatibility can evolve. Validate behavior in staging before broad production rollout.

??? info "Advanced: `dhash` handling strategy"
    For high-frequency disallowed-list writes, keep the latest successful `dhash` per `PrivacyCategory`.
    If server-side conflict appears, refresh settings and retry once with an updated list snapshot.

## Related Docs

- [Types API](types.md)
- [Profile Namespace](profile.md)
- [Tutorial: Profile and Privacy](../tutorials/profile-privacy.md)
