# Community Namespace (`client.community`)

Use `client.community` to create and manage communities, link subgroups, and query topology details.

!!! tip "Data model"
    Community operations are graph-like: one parent community can own multiple linked groups with different participant sets.

## Method Matrix

| Method | Purpose |
| --- | --- |
| `classify_group(metadata)` | Determine logical group type |
| `create(options)` | Create a community |
| `deactivate(community_jid)` | Deactivate community |
| `link_subgroups(community_jid, subgroup_jids)` | Attach existing groups |
| `unlink_subgroups(community_jid, subgroup_jids, remove_orphan_members)` | Detach linked groups |
| `get_subgroups(community_jid)` | List linked subgroup metadata |
| `get_subgroup_participant_counts(community_jid)` | Participant count by subgroup |
| `query_linked_group(community_jid, subgroup_jid)` | Query one linked subgroup |
| `join_subgroup(community_jid, subgroup_jid)` | Join a subgroup through community context |
| `get_linked_groups_participants(community_jid)` | Aggregate participants |

## Runnable Example: Create and Link

```python
from tryx.client import CreateCommunityOptions


async def bootstrap_community(client, subgroup_jids):
    options = CreateCommunityOptions(
        name="Engineering Org",
        description="Platform and Product teams",
        closed=False,
        allow_non_admin_sub_group_creation=False,
        create_general_chat=True,
    )

    created = await client.community.create(options)
    result = await client.community.link_subgroups(created.gid, subgroup_jids)
    return created.gid, result.linked_jids, result.failed_groups
```

## Runnable Example: Topology Audit

```python
async def audit_topology(client, community_jid):
    groups = await client.community.get_subgroups(community_jid)
    counts = await client.community.get_subgroup_participant_counts(community_jid)
    by_jid = {jid.user: count for jid, count in counts}

    report = []
    for group in groups:
        report.append(
            {
                "id": group.id.user,
                "subject": group.subject,
                "participants": by_jid.get(group.id.user, group.participant_count),
                "is_general": group.is_general_chat,
            }
        )
    return report
```

!!! warning "Link failures"
    `LinkSubgroupsResult.failed_groups` and `UnlinkSubgroupsResult.failed_groups` are first-class outcomes.
    Always inspect them and avoid assuming all subgroup operations succeeded.

## Related Docs

- [Groups Namespace](groups.md)
- [Types API](types.md)
- [Tutorial: Group Automation](../tutorials/group-automation.md)
