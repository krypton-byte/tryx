# Tutorial: Group Automation

## Common Tasks

- create groups
- update subject/description
- add/remove/promote/demote participants
- configure announce/locked/ephemeral modes

## Pattern

1. Read current metadata (`get_metadata`)
2. Compute minimal change
3. Apply targeted action
4. Handle sync events (`EvGroupUpdate`) for confirmation

## Example Action

```python
await client.groups.set_subject(group_jid, "Engineering Room")
await client.groups.set_announce(group_jid, True)
```

## Safety Rules

- Check bot admin privileges before mutating group state.
- Use idempotent wrappers to avoid repeated side effects.
- Keep audit logs for participant management operations.
