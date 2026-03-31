# Helpers API

Helpers in `tryx.helpers` are stateless utility surfaces for builders, enum defaults, and payload conversion.

!!! tip "Design intent"
	Use helpers for deterministic object construction and conversion logic. Keep side effects in client namespace calls.

## NewsletterHelpers

| Method | Return |
| --- | --- |
| `parse_message(data)` | `MessageProto` |
| `serialize_message(message)` | `bytes` |
| `build_text_message(text)` | `MessageProto` |

```python
from tryx.helpers import NewsletterHelpers

proto = NewsletterHelpers.build_text_message("Release update")
blob = NewsletterHelpers.serialize_message(proto)
restored = NewsletterHelpers.parse_message(blob)
```

## GroupsHelpers

| Method | Purpose |
| --- | --- |
| `strip_invite_url(code)` | Normalize invite URL/code |
| `build_participant(...)` | Build `GroupParticipantOptions` |
| `build_create_options(...)` | Build `CreateGroupOptions` |

```python
from tryx.helpers import GroupsHelpers

participant = GroupsHelpers.build_participant(jid=target_jid)
opts = GroupsHelpers.build_create_options(subject="Ops Room", participants=[participant])
result = await client.groups.create_group(opts)
```

## StatusHelpers

| Method | Return |
| --- | --- |
| `build_send_options(privacy=...)` | `StatusSendOptions` |
| `default_privacy()` | `StatusPrivacySetting` |

```python
from tryx.helpers import StatusHelpers

options = StatusHelpers.build_send_options()
await client.status.send_text("status", 0xFF1F9D86, 1, recipients, options=options)
```

## ChatstateHelpers

| Method | Return |
| --- | --- |
| `composing()` | `ChatStateType.Composing` |
| `recording()` | `ChatStateType.Recording` |
| `paused()` | `ChatStateType.Paused` |

```python
from tryx.helpers import ChatstateHelpers

await client.chatstate.send(chat_jid, ChatstateHelpers.composing())
```

## BlockingHelpers

| Method | Return |
| --- | --- |
| `same_user(a, b)` | `bool` |

```python
from tryx.helpers import BlockingHelpers

if BlockingHelpers.same_user(a, b):
	print("Equivalent identity")
```

## PollsHelpers

| Method | Return |
| --- | --- |
| `decrypt_vote(...)` | `list[bytes]` |
| `aggregate_votes(...)` | `list[PollOptionResult]` |

```python
from tryx.helpers import PollsHelpers

decoded = PollsHelpers.decrypt_vote(enc_payload, enc_iv, secret, poll_id, creator_jid, voter_jid)
```

## PresenceHelpers

| Method | Return |
| --- | --- |
| `default_status()` | `PresenceStatus` |

```python
from tryx.helpers import PresenceHelpers

await client.presence.set(PresenceHelpers.default_status())
```

## Integration Map

=== "Builder Heavy"
	Use helpers before calling:
	- [Groups Namespace](groups.md)
	- [Status Namespace](status.md)

=== "Crypto and Payload"
	Use helpers with:
	- [Polls Namespace](polls.md)
	- [Newsletter Namespace](newsletter.md)

=== "Signal Defaults"
	Use helpers with:
	- [Chatstate Namespace](chatstate.md)
	- [Presence Namespace](presence.md)

!!! warning "Avoid mixing concerns"
	Helpers should not replace runtime validation. Keep business rules in handler/service layers.
