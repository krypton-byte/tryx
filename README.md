# Tryx

Tryx is a Rust-powered Python SDK for building WhatsApp automations with an async-first developer experience, typed APIs, and high runtime efficiency.

It combines:
- Rust for protocol, transport, and runtime-heavy work
- PyO3 for Python bindings
- Tokio for async orchestration
- Protobuf interop via generated WhatsApp Python types

## Why Tryx

- Low-latency runtime path for event processing
- Python-friendly API surface for application logic
- Structured event model with explicit classes
- Optional blocking mode for script-style execution
- Typed package distribution with `.pyi` and `py.typed`

## Key Features

- Async bot lifecycle: `await bot.run()`
- Blocking lifecycle for simple scripts: `bot.run_blocking()`
- Event registration decorator: `@bot.on(EventType)`
- Messaging API (text, media upload, media download)
- Dedicated contact namespace: `client.contact.*`
- Dedicated chat-actions namespace: `client.chat_actions.*`
- Dedicated community namespace: `client.community.*`
- Dedicated newsletter namespace: `client.newsletter.*`
- Dedicated groups namespace: `client.groups.*`
- Dedicated status namespace: `client.status.*`
- Dedicated chatstate namespace: `client.chatstate.*`
- Dedicated blocking namespace: `client.blocking.*`
- Dedicated polls namespace: `client.polls.*`
- Dedicated presence namespace: `client.presence.*`
- Dedicated helper namespace: `tryx.helpers.*`
- Rich event payload classes with lazy conversion where possible

## Architecture Overview

Tryx is split into two layers:

1. Core (Rust)
- Transport, protocol state, and event stream integration
- WhatsApp runtime from submodule stack in `libs/whatsapp-rust`
- PyO3 bindings in `src/`

2. Interface (Python)
- Dynamic re-export modules in `python/tryx/*.py`
- Type stubs in `python/tryx/*.pyi`
- Generated protobuf package in `python/tryx/waproto`

## Native Binding Advantages (Rust + PyO3)

Tryx uses native Rust bindings instead of a pure-Python protocol implementation.
This gives concrete benefits for this specific project:

- Lower CPU overhead on hot paths such as event parsing and media/protobuf conversion.
- Better memory behavior because heavy objects stay in Rust and are exposed to Python only when needed.
- Async safety and runtime control from Tokio while keeping Python application code simple.
- Ability to cache expensive Python type lookups once (PyOnceLock) and reuse them across events.
- Cleaner separation: Rust handles protocol/runtime mechanics, Python handles business logic and integrations.

In practical terms, this means Python callbacks remain expressive while most protocol-heavy work stays fast and predictable.

## Centralized PyOnceLock Cache

Event protobuf type caches are centralized in `src/events/proto_cache.rs`.

Why this helps:
- All static PyOnceLock declarations are in one file.
- All cache lookup helpers are in one place.
- Easier maintenance and code search when adding/removing protobuf-backed fields.
- Lower risk of duplicated cache logic in multiple event files.

The event layer now consumes cache helpers from this module, keeping event structs focused on payload mapping instead of cache plumbing.

## Concurrency and Overhead Model

Tryx currently uses `watch::Receiver<Option<Arc<Client>>>` to expose the active client across binding objects.

Why this is a good default for PyO3 async bindings:
- `watch::Receiver` is read-optimized and cheap to clone.
- Stored value is `Arc<Client>`, so clone cost is minimal (atomic refcount).
- Works naturally with Tokio async context.
- Avoids explicit lock management in Python-exposed methods.

Compared to `RwLock<Option<Arc<Client>>>`:
- `RwLock` adds lock acquisition on every read path.
- It can increase contention under frequent method calls.
- In mixed Python/Rust workloads, lock handoff can be noisier than `watch` read snapshots.

Recommendation:
- Keep `watch::Receiver<Option<Arc<Client>>>` for low overhead and async safety.
- Use `RwLock` only if you need mutable shared state beyond swapping client snapshots.

## Contact Client Design

Tryx now exposes contact APIs through a dedicated `ContactClient` pyclass:

- `client.contact.get_info(...)`
- `client.contact.get_user_info(...)`
- `client.contact.get_profile_picture(...)`
- `client.contact.is_on_whatsapp(...)`

This keeps `TryxClient` focused on messaging/media and keeps contacts grouped by responsibility with no extra heavy synchronization cost.

## Project Structure

- `src/lib.rs`: PyO3 module bootstrap and submodule registration
- `src/clients/tryx.rs`: main `Tryx` runtime wrapper
- `src/clients/tryx_client.rs`: messaging/media client methods
- `src/clients/contacts.rs`: contact-specific client methods
- `src/events/`: dispatcher and event payload classes
- `src/types.rs`: core Python-exposed value types (`JID`, `MessageInfo`, ...)
- `python/tryx/`: Python package surface and stubs
- `python/tryx/waproto/`: generated protobuf Python files
- `libs/whatsapp-rust/`: embedded rust stack dependencies

## Installation

### Prerequisites

- Python 3.8+
- Rust stable toolchain
- `pip` and virtual environment tooling

### Development install (editable)

```bash
python -m venv .venv
source .venv/bin/activate
pip install -U pip maturin
maturin develop
```

### Build wheel

```bash
maturin build --release
```

Wheels are produced under `target/wheels` or project-specific wheel output depending on command options.

## Quick Start

```python
import asyncio
from tryx.backend import SqliteBackend
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage
from tryx.waproto.whatsapp_pb2 import Message

backend = SqliteBackend("whatsapp.db")
bot = Tryx(backend)

@bot.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = event.data.get_text() or "<non-text>"
    chat = event.data.message_info.source.chat
    await client.send_message(chat, Message(conversation=f"Echo: {text}"))

async def main() -> None:
    await bot.run()

if __name__ == "__main__":
    asyncio.run(main())
```

## Python API Reference (High Level)

### Backend

- `SqliteBackend(path: str)`

### Bot controller

- `Tryx(backend)`
- `Tryx.on(event_type)`
- `await Tryx.run()`
- `Tryx.run_blocking()`
- `Tryx.get_client() -> TryxClient`

### Runtime client

- `TryxClient.contact -> ContactClient`
- `TryxClient.chat_actions -> ChatActionsClient`
- `TryxClient.community -> CommunityClient`
- `TryxClient.newsletter -> NewsletterClient`
- `TryxClient.groups -> GroupsClient`
- `TryxClient.status -> StatusClient`
- `TryxClient.chatstate -> ChatstateClient`
- `TryxClient.blocking -> BlockingClient`
- `TryxClient.polls -> PollsClient`
- `TryxClient.presence -> PresenceClient`
- `TryxClient.send_message(...)`
- `TryxClient.send_text(...)`
- `TryxClient.send_photo(...)`
- `TryxClient.download_media(...)`
- `TryxClient.upload(...)`
- `TryxClient.upload_file(...)`

### Contact namespace

- `ContactClient.get_info(phones)`
- `ContactClient.get_user_info(jid)`
- `ContactClient.get_profile_picture(jid, preview)`
- `ContactClient.is_on_whatsapp(jids)`

### Chat actions namespace

- `ChatActionsClient.archive_chat(jid, message_range=None)`
- `ChatActionsClient.unarchive_chat(jid, message_range=None)`
- `ChatActionsClient.pin_chat(jid)`
- `ChatActionsClient.unpin_chat(jid)`
- `ChatActionsClient.mute_chat(jid)`
- `ChatActionsClient.mute_chat_until(jid, mute_end_timestamp_ms)`
- `ChatActionsClient.unmute_chat(jid)`
- `ChatActionsClient.star_message(chat_jid, participant_jid, message_id, from_me)`
- `ChatActionsClient.unstar_message(chat_jid, participant_jid, message_id, from_me)`
- `ChatActionsClient.mark_chat_as_read(jid, read, message_range=None)`
- `ChatActionsClient.delete_chat(jid, delete_media, message_range=None)`
- `ChatActionsClient.delete_message_for_me(chat_jid, participant_jid, message_id, from_me, delete_media, message_timestamp=None)`
- `ChatActionsClient.build_message_key(...)`
- `ChatActionsClient.build_message_range(...)`

### Community namespace

- `CommunityClient.create(options)`
- `CommunityClient.deactivate(community_jid)`
- `CommunityClient.link_subgroups(community_jid, subgroup_jids)`
- `CommunityClient.unlink_subgroups(community_jid, subgroup_jids, remove_orphan_members)`
- `CommunityClient.get_subgroups(community_jid)`
- `CommunityClient.get_subgroup_participant_counts(community_jid)`
- `CommunityClient.query_linked_group(community_jid, subgroup_jid)`
- `CommunityClient.join_subgroup(community_jid, subgroup_jid)`
- `CommunityClient.get_linked_groups_participants(community_jid)`

### Newsletter namespace

- `NewsletterClient.list_subscribed()`
- `NewsletterClient.get_metadata(jid)`
- `NewsletterClient.get_metadata_by_invite(invite_code)`
- `NewsletterClient.create(name, description=None)`
- `NewsletterClient.join(jid)`
- `NewsletterClient.leave(jid)`
- `NewsletterClient.update(jid, name=None, description=None)`
- `NewsletterClient.subscribe_live_updates(jid)`
- `NewsletterClient.send_message(jid, message)`
- `NewsletterClient.send_reaction(jid, server_id, reaction)`
- `NewsletterClient.get_messages(jid, count, before=None)`

### Groups namespace

- `GroupsClient.query_info(jid)`
- `GroupsClient.get_participating()`
- `GroupsClient.get_metadata(jid)`
- `GroupsClient.create_group(options)`
- `GroupsClient.set_subject(jid, subject)`
- `GroupsClient.set_description(jid, description=None, prev=None)`
- `GroupsClient.leave(jid)`
- `GroupsClient.add_participants(jid, participants)`
- `GroupsClient.remove_participants(jid, participants)`
- `GroupsClient.promote_participants(jid, participants)`
- `GroupsClient.demote_participants(jid, participants)`
- `GroupsClient.get_invite_link(jid, reset)`
- `GroupsClient.set_locked(jid, locked)`
- `GroupsClient.set_announce(jid, announce)`
- `GroupsClient.set_ephemeral(jid, expiration)`
- `GroupsClient.set_membership_approval(jid, mode)`
- `GroupsClient.join_with_invite_code(code)`
- `GroupsClient.join_with_invite_v4(group_jid, code, expiration, admin_jid)`
- `GroupsClient.get_invite_info(code)`
- `GroupsClient.get_membership_requests(jid)`
- `GroupsClient.approve_membership_requests(jid, participants)`
- `GroupsClient.reject_membership_requests(jid, participants)`
- `GroupsClient.set_member_add_mode(jid, mode)`

### Status namespace

- `StatusClient.send_text(text, background_argb, font, recipients, options=None)`
- `StatusClient.send_image(upload, thumbnail, recipients, caption=None, options=None)`
- `StatusClient.send_video(upload, thumbnail, duration_seconds, recipients, caption=None, options=None)`
- `StatusClient.send_raw(message, recipients, options=None)`
- `StatusClient.revoke(message_id, recipients, options=None)`

### Chatstate namespace

- `ChatstateClient.send(to, state)`
- `ChatstateClient.send_composing(to)`
- `ChatstateClient.send_recording(to)`
- `ChatstateClient.send_paused(to)`

### Blocking namespace

- `BlockingClient.block(jid)`
- `BlockingClient.unblock(jid)`
- `BlockingClient.get_blocklist()`
- `BlockingClient.is_blocked(jid)`

### Polls namespace

- `PollsClient.create(to, name, options, selectable_count)`
- `PollsClient.vote(chat_jid, poll_msg_id, poll_creator_jid, message_secret, option_names)`
- `PollsClient.decrypt_vote(enc_payload, enc_iv, message_secret, poll_msg_id, poll_creator_jid, voter_jid)`
- `PollsClient.aggregate_votes(poll_options, votes, message_secret, poll_msg_id, poll_creator_jid)`

### Presence namespace

- `PresenceClient.set(status)`
- `PresenceClient.set_available()`
- `PresenceClient.set_unavailable()`
- `PresenceClient.subscribe(jid)`
- `PresenceClient.unsubscribe(jid)`

### Helper namespace

- `NewsletterHelpers.parse_message(data)`
- `NewsletterHelpers.serialize_message(message)`
- `NewsletterHelpers.build_text_message(text)`
- `GroupsHelpers.strip_invite_url(code)`
- `GroupsHelpers.build_participant(...)`
- `GroupsHelpers.build_create_options(...)`
- `StatusHelpers.build_send_options(privacy=...)`
- `StatusHelpers.default_privacy()`
- `ChatstateHelpers.composing()`
- `ChatstateHelpers.recording()`
- `ChatstateHelpers.paused()`
- `BlockingHelpers.same_user(a, b)`
- `PollsHelpers.decrypt_vote(...)`
- `PollsHelpers.aggregate_votes(...)`
- `PresenceHelpers.default_status()`

Related typed models:

- `CreateCommunityOptions`
- `CreateCommunityResult`
- `CommunitySubgroup`
- `LinkSubgroupsResult`
- `UnlinkSubgroupsResult`
- `GroupParticipant`
- `GroupMetadata`
- `GroupType`
- `NewsletterVerification`
- `NewsletterState`
- `NewsletterRole`
- `NewsletterReactionCount`
- `NewsletterMetadata`
- `NewsletterMessage`
- `MemberLinkMode`
- `MemberAddMode`
- `MembershipApprovalMode`
- `GroupParticipantOptions`
- `CreateGroupOptions`
- `CreateGroupResult`
- `JoinGroupResult`
- `ParticipantChangeResponse`
- `MembershipRequest`
- `GroupInfo`
- `StatusPrivacySetting`
- `StatusSendOptions`
- `ChatStateType`
- `BlocklistEntry`
- `PollOptionResult`
- `PresenceStatus`

## Typing Support

Tryx ships as a typed Python package:
- Stub files in `python/tryx/*.pyi`
- Marker file `python/tryx/py.typed`

Recommended type checkers:
- Pyright
- Mypy
- Pylance

## Events

Event classes are generated from Rust-side event payloads and exposed under `tryx.events`.

Common patterns:
- `event.data` for structured payload
- lazy-converted proto fields (for lower eager conversion overhead)
- datetime and typed references where available

## Error Handling

Tryx exposes binding-level exceptions in `tryx.exceptions`, including:
- `FailedBuildBot`
- `EventDispatchError`
- `UnsupportedBackend`
- `UnsupportedEventType`

Backward-compatible aliases are also available for older names.

## Development Workflow

### Rust checks

```bash
cargo check
```

### Python package sanity

```bash
python -c "import tryx; print('ok')"
```

### Type checking example

```bash
pyright
# or
mypy examples.py
```

## Performance Notes

- Avoid creating Python objects before `.await` points in Rust async methods.
- Construct Python values inside `Python::attach(...)` after async IO completes.
- Return owned `Py<T>` from futures when required by `Send` bounds.
- Keep payload conversion lazy when field access is infrequent.
- Centralize Python type/proto caches to minimize repeated import/lookups.

## Troubleshooting

### Import error for native module

Symptom:
- `ModuleNotFoundError: No module named 'tryx._tryx'`

Fix:

```bash
maturin develop --release
```

### Bot is not running

Symptom:
- Python methods raise runtime error before run/start

Fix:
- Ensure bot is started (`run`/`run_blocking`) and connected before invoking runtime client methods.

### Type checker not reading stubs

Fix:
- Ensure local install is active in your environment
- Confirm `py.typed` is included in installed package
- Restart language server

## Security and Compliance

- Keep secrets and session files outside version control
- Use WhatsApp automation responsibly and within platform policy
- Audit message handling callbacks before deploying production bots

## License

See `LICENSE` for license terms.
