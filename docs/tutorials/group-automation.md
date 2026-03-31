# Tutorial: Group Automation

Automate full group lifecycle with safe participant governance.

## Common Workflows

- create and configure group
- manage participants (add/remove/promote/demote)
- handle membership approvals
- keep chat policy aligned (announce/locked/ephemeral)

## Basic: Create + Configure

```python
from tryx.client import CreateGroupOptions, GroupParticipantOptions


participants = [GroupParticipantOptions(jid=jid) for jid in initial_members]
options = CreateGroupOptions(subject="Engineering Room", participants=participants)
created = await client.groups.create_group(options)

await client.groups.set_subject(created.gid, "Engineering Room")
await client.groups.set_announce(created.gid, True)
await client.groups.set_locked(created.gid, True)
```

## Intermediate: Membership Queue

```python
pending = await client.groups.get_membership_requests(group_jid)

approve = [row.jid for row in pending if row.jid.user in trusted_users]
reject = [row.jid for row in pending if row.jid.user not in trusted_users]

if approve:
	await client.groups.approve_membership_requests(group_jid, approve)
if reject:
	await client.groups.reject_membership_requests(group_jid, reject)
```

## Production: State Reconciliation Loop

```python
async def reconcile_group(client, group_jid):
	metadata = await client.groups.get_metadata(group_jid)

	# enforce policy
	if not metadata.is_announcement:
		await client.groups.set_announce(group_jid, True)

	if metadata.ephemeral_expiration != 604800:
		await client.groups.set_ephemeral(group_jid, 604800)
```

## Operational Safety Rules

!!! warning
	Always verify the bot has required admin privileges before participant mutation.

!!! tip
	Write audit logs for every moderator action: actor, target group, participant, action, timestamp.

## Related Docs

- [Groups Namespace](../api/groups.md)
- [Community Namespace](../api/community.md)
- [Troubleshooting](../operations/troubleshooting.md)
