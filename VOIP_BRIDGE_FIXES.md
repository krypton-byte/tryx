# Experimental WhatsApp VoIP fixes (Tryx 1.5.0)

This branch is based on the upstream Tryx 1.5.0 release commit
`646dae2883522e74db6e665ce4afddcf9c4f3be6`. It contains native
changes required for a Linux WhatsApp ↔ SIP gateway. The fork's `master`
is not modified.

## Changes

1. **Tokio audio adapters** (`src/voip.rs`): the Python-source and
   Python-sink PCM adapter tasks use
   `pyo3_async_runtimes::tokio::get_runtime().spawn(...)` instead of
   `tokio::spawn(...)`. The Python event loop does not enter the Rust
   Tokio reactor, so bare spawns from `IncomingCallEvent.accept()`
   previously caused a PyO3 `PanicException`.

2. **Native media relay and Opus** (`Cargo.toml`): select
   `whatsapp-rust/voip-relay-native` so Linux can connect to WhatsApp
   UDP media relays. Enable `voip-libopus` so receiving ordinary Opus
   audio works alongside the existing MLOW support. The Linux builder
   requires `libopus-dev`; the runtime requires `libopus0`.

3. **Remote hangup** (`src/clients/voip.rs`):
   `CallHandle.hangup()` calls `terminate()` rather than
   `hangup_local()`. The former sends a WhatsApp `<terminate>`
   signaling message, while the latter stops only local media and leaves
   the peer's call active.

4. **Actual outgoing answer state** (`src/clients/voip.rs`,
   `src/clients/tryx.rs`, and `python/tryx/client.pyi`):
   `VoipClient.is_call_answered(call_id)` reads a small, per-client
   set of call IDs updated by the **existing public WhatsApp call-event
   stream** when the peer sends `<accept>`. This avoids
   `Client::call_registry()`, which is `pub(crate)` inside the
   dependency and inaccessible to Tryx (Rust E0624). Terminal remote
   events and disconnect clear the relevant state, with the set capped
   at 64 entries. Use the getter for active calls; a returned
   `CallHandle` alone does not mean the remote user answered.

5. **Correct call event dispatch** (`src/clients/tryx.rs`):
   only real `<offer>` stanzas become Python `IncomingCallEvent`
   callbacks. Previously, outbound call `<accept>` events could be
   mistaken for second incoming offers and rejected.

## Testing

The experimental bridge builds Tryx from this branch in its
`Dockerfile.tryx-calls`, installs the resulting native wheel and smoke
checks the `VoipClient.is_call_answered` API. The bridge's Python tests
cover SIP ring/answer/reject, no-answer, hangup, and media behavior.
Real call behavior must be verified with two test devices; native
compilation alone cannot validate WhatsApp's signaling.

## Upstream contribution notes

For an upstream PR, consider separating **bug fixes** (Tokio adapter task
spawns, remote hangup) from **features** (native relay/Opus build flags and
Python answer-state API). Review native feature defaults with upstream
maintainers; enabling platform-specific relay and Opus for every build
may not be appropriate for non-Linux deployments.

Do not include bridge-specific FreeSWITCH configuration in the Tryx
upstream PR. The transport and API fixes belong to Tryx; SIP-specific
call control belongs to the bridge.
