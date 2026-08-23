# Glossary

---

## Core Concepts

### JID
**Jabber ID** — A WhatsApp identifier containing user and server segments.
Examples: `5599800001@s.whatsapp.net` (personal), `120363000000000000@g.us`
(group), `120363000000000000@newsletter` (channel).

### LID
**Linked ID** — An alternate addressing identity used in multi-device
contexts. Maps between phone number JIDs and linked device identities.

### Signal Protocol
The end-to-end encryption protocol used by WhatsApp. Tryx implements
Double Ratchet and X3DH key exchange in Rust.

### Noise Protocol
The transport encryption protocol used for WebSocket connections between
Tryx and WhatsApp servers.

---

## Architecture

### Dispatcher
Internal callback registry mapping event classes to Python functions.
Registered via `@app.on(EventClass)`.

### PyO3
The Rust crate used to create Python bindings. Tryx uses PyO3 to bridge
Rust protocol logic with Python application code.

### WACore
Low-level protocol-oriented module exposing node and stanza structures
for direct protocol interaction.

### Protobuf
Protocol Buffers — the binary serialization format used by WhatsApp for
message encoding. Tryx uses `whatsapp.proto` for (de)serialization.

---

## Events

### EvMessage
Main incoming message event class. Contains message metadata, content,
and media information.

### Sync Action
State update events propagated from server/app-state, such as mute,
archive, delete, and read status changes.

### EvStreamReplaced
Event indicating another session has replaced your active WebSocket stream.

### EvLoggedOut
Event indicating the session is no longer valid and re-pairing is needed.

---

## Storage

### Backend
The persistence layer for Signal protocol state. Tryx supports three
tiers: SQLite (built-in), FFI (native shared library), and Python (async).

### FFI
**Foreign Function Interface** — A C ABI shared library loaded via
`libloading`. Used for high-performance storage backends like Postgres.

### StoreBase
Abstract base class for pure Python storage backends. Implement all
abstract methods to create custom backends.

---

## Messaging

### Newsletter
Channel-like broadcast construct exposed through `client.newsletter`.
Supports subscription, posting, and follower management.

### Media Reupload
Workflow to refresh media retrieval paths for expired media references.
Use `request_media_reupload()` when normal download fails.

### Push-to-Talk (PTT)
Voice message mode. Set `ptt=True` in `send_audio()` to send as a
voice note rather than a regular audio clip.

---

## Groups

### Announcement Group
A group where only admins can send messages. Set via
`client.groups.set_announce(jid, True)`.

### Membership Approval
Mode requiring admin approval for new members. Set via
`client.groups.set_membership_approval(jid, MembershipApprovalMode.On)`.

### Ephemeral Messages
Disappearing message timer. Set via
`client.groups.set_ephemeral(jid, seconds)`. Use 0 to disable.

---

## Deployment

### Session State
Persistent data from the Signal protocol handshake. Stored in the
backend and reused across restarts. Never delete unless intentional.

### Single Writer
Rule: only one runtime instance should write to a given backend path.
Multiple writers cause stream replacement and data corruption.

### Blue/Green Rollout
Deployment strategy where two identical environments (blue and green)
take turns serving traffic. During rollout, ensure only one writes to
the session backend.
