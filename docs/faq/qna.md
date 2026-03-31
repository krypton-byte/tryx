# QnA

## General

### What is Tryx?
Tryx is a Rust-powered Python SDK for event-driven WhatsApp automation.

### Why not pure Python?
Rust handles protocol-heavy runtime work for better throughput and lower overhead, while Python keeps app logic easy to write.

### Is Tryx synchronous or asynchronous?
Both: async-first (`await bot.run()`), plus blocking convenience (`bot.run_blocking()`).

## Pairing and Session

### Do I need to pair every time?
No. If backend storage is preserved, session data is reused.

### What does EvStreamReplaced mean?
Another session replaced your active stream. Re-check device/session ownership.

### What should I do on EvLoggedOut?
Treat it as session invalidation. Re-pair and refresh persisted state.

## Event Handling

### Can I register multiple handlers for one event?
Yes. Dispatcher stores callbacks per event class.

### Why does an event have `data` property instead of direct fields?
Many event payloads are lazily materialized for efficiency.

### Should I process heavy logic directly in handlers?
Prefer short handlers that delegate expensive work to background tasks.

## Messaging and Media

### Which media types can Tryx send?
Text, photo, document, audio, video, GIF, sticker, and protobuf-raw messages.

### When should I call request_media_reupload?
When media direct path is stale or unavailable and normal download fails.

### Can I quote a message in replies?
Yes, pass the original `EvMessage` to send helpers that support `quoted`.

## Groups and Privacy

### Can I automate group moderation?
Yes, use `client.groups.*` and handle `EvGroupUpdate` for state feedback.

### Can I modify privacy settings?
Yes, use `client.privacy.fetch_settings()` and `set_setting(...)`.

## Typing and Tooling

### Are stubs complete?
Tryx ships `.pyi` stubs for public modules including events and low-level wacore types.

### Can I use mypy or pyright?
Yes, the package includes `py.typed` for static analysis integration.

## Reliability

### How should I handle temporary bans?
Listen to `EvTemporaryBan`, pause high-frequency operations, and avoid aggressive retries.

### How can I make my bot idempotent?
Store processed message IDs and guard side effects before calling external systems.

## Compatibility

### Which Python versions are supported?
Python 3.8 and newer.

### Does Tryx support Linux/macOS/Windows?
Yes, with proper Rust toolchain and platform build dependencies.
