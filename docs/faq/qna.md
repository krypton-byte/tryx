# :material-frequently-asked-questions: FAQ

Quick answers to common questions. Jump to the linked pages for implementation
details.

---

## General

### What is Tryx?

Tryx is a Rust-powered Python SDK for event-driven WhatsApp automation. It
pairs a Rust runtime core with a typed Python API, giving you WhatsApp
protocol handling at native speed while keeping application logic in clean
async Python.

### Why not pure Python?

Rust handles protocol-heavy runtime work for better throughput and lower
overhead:

- **Signal protocol** — Double Ratchet, X3DH key exchange
- **Noise handshake** — WebSocket transport encryption
- **Protobuf** — Message serialization/deserialization
- **Media processing** — Upload/download, transcoding

Python keeps app logic easy to write and maintain.

### Is Tryx synchronous or asynchronous?

Both:

- **Async-first**: `await app.run()` with full asyncio support
- **Blocking**: `app.run_blocking()` for quick scripts

```python
# Async
asyncio.run(app.run())

# Blocking
app.run_blocking()
```

See [Quick Start](../getting-started/quickstart.md).

### What Python versions are supported?

Python 3.8 and newer. We recommend 3.10+ for the best typing experience.

### Does Tryx support Linux/macOS/Windows?

Yes, with proper Rust toolchain and platform build dependencies. See the
[Installation](../getting-started/installation.md) page for platform-specific
setup.

---

## Pairing and Session

### Do I need to pair every time?

No. If backend storage is preserved (`whatsapp.db` or equivalent), session
data is reused across restarts.

### What does `EvStreamReplaced` mean?

Another session replaced your active stream. This typically happens when:

- Another device logged into the same account
- A deployment is using the same backend path

Check device/session ownership and ensure single-writer backend access.

### What should I do on `EvLoggedOut`?

Treat it as session invalidation. Re-pair and refresh persisted state. See
[Authentication Flow](../getting-started/authentication.md).

### Can I use the same backend path across multiple instances?

**No.** Avoid multiple runtime instances writing to the same backend path.
This can cause stream replacement, data corruption, or forced re-pairing.

---

## Event Handling

### Can I register multiple handlers for one event?

Yes. The dispatcher stores callbacks per event class and calls all of them
for each event.

### Why does an event have a `data` property instead of direct fields?

Many event payloads are lazily materialized for efficiency. The `data`
property returns a rich typed object that's cached for subsequent access.

### Should I process heavy logic directly in handlers?

**No.** Keep handlers short and non-blocking. Delegate expensive work to
background tasks. See [Reliability](../operations/reliability.md) and
[Performance](../operations/performance.md).

### How do I handle undecryptable messages?

Treat them as normal — they happen during key rotation. Log them and move
on. Tryx continues dispatching other events.

---

## Messaging and Media

### Which media types can Tryx send?

| Type | Method | Notes |
|------|--------|-------|
| Text | `send_text()` | Basic text messages |
| Photo | `send_photo()` | Images with optional caption |
| Document | `send_document()` | Files with optional name |
| Audio | `send_audio()` | Voice notes (ptt=True) or clips |
| Video | `send_video()` | Video clips with optional caption |
| GIF | `send_gif()` | Animated GIFs |
| Sticker | `send_sticker()` | Static WEBP or animated |
| Raw | `send_message()` | Custom protobuf messages |

### When should I call `request_media_reupload`?

When media direct path is stale or unavailable and normal download fails.
This is common for older messages where the media CDN link has expired.

### Can I quote a message in replies?

Yes, pass the original `EvMessage` to send helpers that support `quoted`:

```python
await client.send_text(chat, "reply text", quoted=event)
```

See [Media Workflows](../tutorials/media-workflows.md).

---

## Groups and Privacy

### Can I automate group moderation?

Yes, use `client.groups.*` for participant management and handle
`EvGroupUpdate` for state feedback. See
[Group Automation](../tutorials/group-automation.md).

### Can I modify privacy settings?

Yes, use `client.privacy.fetch_settings()` and `set_setting(...)`. See
[Privacy Namespace](../api/privacy.md).

---

## Deployment and Operations

### What is the minimum production checklist?

1. Durable backend/session storage
2. Bounded retry strategy
3. Idempotent message processing
4. Basic security controls (admin-only commands, secret management)
5. Structured logging

See [Deployment Guide](../operations/deployment.md).

### How do I troubleshoot reconnect loops?

Use the connection decision tree in
[Troubleshooting](../operations/troubleshooting.md) and verify single-writer
backend ownership.

---

## Typing and Tooling

### Are stubs complete?

Tryx ships `.pyi` stubs for all public modules including events, types,
client namespaces, and low-level wacore types.

### Can I use mypy or pyright?

Yes, the package includes `py.typed` for static analysis integration.

```bash
uv run mypy your_project/
uv run pyright your_project/
```

---

## Reliability

### How should I handle temporary bans?

Listen to `EvTemporaryBan`, pause high-frequency operations, and avoid
aggressive retries. Resume gradually after the ban period.

### How can I make my client idempotent?

Store processed message IDs and guard side effects before calling external
systems. See [Reliability](../operations/reliability.md).
