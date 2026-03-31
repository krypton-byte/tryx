# Groups Namespace (`client.groups`)

`client.groups` covers group lifecycle, membership controls, invite flows, and moderation primitives.

!!! note "Scope"
    Use this namespace for normal groups. For community-parent orchestration, pair this page with [Community Namespace](community.md).

## Method Families

=== "Lifecycle"
    - `create_group(options)`
    - `leave(jid)`
    - `query_info(jid)`
    - `get_participating()`
    - `get_metadata(jid)`

=== "Metadata"
    - `set_subject(jid, subject)`
    - `set_description(jid, description=None, prev=None)`
    - `set_locked(jid, locked)`
    - `set_announce(jid, announce)`
    - `set_ephemeral(jid, expiration)`
    - `set_member_add_mode(jid, mode)`
    - `set_membership_approval(jid, mode)`

=== "Participants"
    - `add_participants(jid, participants)`
    - `remove_participants(jid, participants)`
    - `promote_participants(jid, participants)`
    - `demote_participants(jid, participants)`
    - `get_membership_requests(jid)`
    - `approve_membership_requests(jid, participants)`
    - `reject_membership_requests(jid, participants)`

=== "Invites"
    - `get_invite_link(jid, reset)`
    - `join_with_invite_code(code)`
    - `join_with_invite_v4(group_jid, code, expiration, admin_jid)`
    - `get_invite_info(code)`

## Runnable Example: Group Setup Playbook

```python
from tryx.client import CreateGroupOptions, GroupParticipantOptions, MembershipApprovalMode


async def setup_group(client, title, member_jids):
    participants = [GroupParticipantOptions(jid=jid) for jid in member_jids]
    options = CreateGroupOptions(subject=title, participants=participants)

    created = await client.groups.create_group(options)
    gid = created.gid

    await client.groups.set_announce(gid, True)
    await client.groups.set_membership_approval(gid, MembershipApprovalMode.On)
    await client.groups.set_ephemeral(gid, 604800)
    return gid
```

## Runnable Example: Membership Queue Handler

```python
async def process_requests(client, group_jid):
    pending = await client.groups.get_membership_requests(group_jid)
    if not pending:
        return {"approved": 0, "rejected": 0}

    approve = [row.jid for row in pending[:10]]
    reject = [row.jid for row in pending[10:]]

    await client.groups.approve_membership_requests(group_jid, approve)
    if reject:
        await client.groups.reject_membership_requests(group_jid, reject)

    return {"approved": len(approve), "rejected": len(reject)}
```

!!! warning "Partial success is valid"
    Participant mutations can return per-user status via `ParticipantChangeResponse`.
    Always inspect each response item and do not treat list-returning calls as all-or-nothing.

## Related Docs

- [Community Namespace](community.md)
- [Helpers API](helpers.md)
- [Tutorial: Group Automation](../tutorials/group-automation.md)
