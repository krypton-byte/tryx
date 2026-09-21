<div align="center">
<img src="assets/mascot.png" width="20%" alt="Tryx">

# Tryx

[![PyPI version](https://img.shields.io/pypi/v/tryx?color=blue)](https://pypi.org/project/tryx/)
[![Python](https://img.shields.io/pypi/pyversions/tryx.svg)](https://pypi.org/project/tryx/)
[![License](https://img.shields.io/github/license/krypton-byte/tryx)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-online-brightgreen)](https://krypton-byte.github.io/tryx/)

**Language:** English | [简体中文](README.zh.md) | [Bahasa Indonesia](README.id.md)

</div>

Tryx is a Python automation library powered by Rust and PyO3. It provides a Python-friendly API on top of the `whatsapp-rust` crate, with async support, persistent sessions, and access to advanced WhatsApp Web protocol features.

> Tryx is an independent project. It is not affiliated with WhatsApp, Meta, or their official products.

## What Is Tryx?

Tryx helps Python developers build WhatsApp automation tools without writing Rust directly. The core protocol logic runs in Rust for performance, while the public interface stays ergonomic for Python applications.

Use Tryx when you need:

- WhatsApp Web automation from Python.
- Persistent login sessions with SQLite, PostgreSQL, or MySQL.
- Async APIs for bots, workers, dashboards, and internal tools.
- Access to contacts, groups, newsletters, status, privacy, profile, labels, comments, events, and lower-level protocol helpers.

## Highlights

- **Rust core, Python API** — protocol logic is implemented in Rust and exposed through PyO3.
- **Async first** — designed for `asyncio` applications.
- **Persistent sessions** — reuse device sessions through a database-backed store.
- **Broad API surface** — exposes common and advanced client namespaces.
- **Typed Python stubs** — includes `.pyi` files for better editor support.

## Installation

```bash
pip install tryx
```

For local development:

```bash
git clone https://github.com/krypton-byte/tryx.git
cd tryx
git submodule update --init --recursive
uv sync --group dev
uv run maturin develop
```

## Quick Start

```python
import asyncio

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)


@app.on(EvPairingQrCode)
async def on_pairing_qr(_client: TryxClient, event: EvPairingQrCode) -> None:
    print("Scan this QR code with WhatsApp:")
    print(event.code)


@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = (event.data.get_text() or "").strip()
    if text.lower() == "ping":
        await client.send_text(
            event.data.message_info.source.chat, "pong", quoted=event
        )


async def main() -> None:
    await app.run()


asyncio.run(main())
```

## Client Namespaces

The runtime `TryxClient` object exposes several namespaces so the API stays organized:

| Namespace | Purpose |
| --- | --- |
| `contact` | Contact lookup and contact-related helpers. |
| `chat_actions` | Chat utilities such as mute, archive, pin, mark read, clear chat, and save contact. |
| `groups` | Group creation, metadata, members, invites, membership requests, and admin settings. |
| `community` | Community-related operations. |
| `newsletter` | Newsletter info, messages, mute settings, edits, and revoke operations. |
| `status` | Status posting, lookup, and related helpers. |
| `chatstate` | Typing, recording, paused, and presence-style chat state updates. |
| `blocking` | Block and unblock contacts. |
| `polls` | Poll creation and voting helpers. |
| `presence` | Presence subscriptions and availability updates. |
| `privacy` | Privacy settings and related controls. |
| `profile` | Profile name, status text, and picture operations. |
| `labels` | Label management for chats and messages. |
| `comments` | Comment-related protocol helpers. |
| `events` | Event response helpers and event-oriented operations. |
| `advanced` | Lifecycle waits, diagnostics, and lower-level protocol escape hatches. |
| `voip` | Audio and video call management. |

## Storage Backends

Tryx supports three storage tiers:

| Tier | Backend | Use case |
| --- | --- | --- |
| Built-in | `SqliteStore("whatsapp.db")` | Local development and simple deployments. |
| Native FFI | `FfiStoreProtocol` | High-throughput external stores such as PostgreSQL. |
| Pure Python | `StoreBase` subclass | Custom async stores such as Redis, MongoDB, or DynamoDB. |

SQLite is usually enough for local development. Native FFI or a custom Python backend is better when several workers or deployment environments need shared session state.

## VoIP: Audio and Video Calls

Tryx provides a Rust-backed bridge for WhatsApp audio and video calls. Protocol handling, RTP/WebRTC, encryption, codecs, and call orchestration run in whatsapp-rust; Python supplies media through asynchronous source and sink adapters.

### Media Contract

| Media | Contract |
| --- | --- |
| Audio | Mono signed PCM16 little-endian, 16,000 Hz, 960 samples / 1,920 bytes per frame, 60 ms |
| Video | H.264 Annex-B access unit represented by `VideoFrame` |

### Minimal Audio Adapters

```python
from collections.abc import AsyncIterator
from tryx.media import AudioSink, AudioSource, validate_audio_frame


class Microphone(AudioSource):
    async def frames(self) -> AsyncIterator[bytes]:
        while True:
            frame = await read_microphone_frame()
            yield validate_audio_frame(frame)


class Speaker(AudioSink):
    async def write(self, frame: bytes) -> None:
        validate_audio_frame(frame)
        await play_speaker_frame(frame)
```

### One-to-One Audio Call

```python
from tryx.types import JID


async def start_audio_call(client, phone_number: str):
    peer = JID(phone_number + "@s.whatsapp.net")
    call = await client.voip.call(peer, Microphone(), Speaker())
    print("call started:", call.call_id)
    call.set_muted(True)
    call.set_muted(False)
    await call.wait_ended()
```

### Video Call

```python
from tryx.media import VideoPlayer


async def start_video_call(client, peer, video_sink):
    video_source = VideoPlayer(fps=15)
    video_source.play("sample.mp4")
    call = await client.voip.video_call(
        peer, Microphone(), Speaker(), video_source, video_sink
    )
    await call.wait_ended()
    video_source.stop()
```

### Group Calls and Call Links

```python
call = await client.voip.group_call(
    peers=[peer_a, peer_b],
    audio_source=Microphone(),
    audio_sink=Speaker(),
)
await call.invite_participant(peer_c)
await call.ring_participant(peer_c)

linked = await client.voip.join_call_link(
    "https://call.whatsapp.com/your-token",
    "audio",
    Microphone(),
    Speaker(),
)
```

### Native AudioPlayer

```python
from tryx.media import AudioPlayer

player = AudioPlayer(buffer_frames=3)
player.play("intro.mp3", mode="replace")
call = await client.voip.call(peer, player, Speaker())
player.pause()
player.resume()
player.enqueue("next.wav")
player.skip()
player.clear_queue()
player.stop()
```

## Development

```bash
cargo check
cargo test --lib
uv run pytest -q
uv run maturin develop
uv run maturin build --release
```

## Project Layout

```text
.
├── libs/whatsapp-rust/   # Rust protocol crate submodule
├── src/                  # Rust and PyO3 bindings
├── python/tryx/          # Python package and type stubs
├── examples/             # Example automation scripts
├── docs/                 # Documentation site
└── tests/                # Python test suite
```

## Links

- Documentation: <https://krypton-byte.github.io/tryx/>
- Examples: [`examples/`](examples/)
- Python type stubs: [`python/tryx/`](python/tryx/)
- Rust bindings: [`src/`](src/)

## License

This project is licensed under the terms of the [MIT License](LICENSE).
