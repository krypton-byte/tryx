# Tryx

Tryx is a Python-first, Rust-powered WhatsApp automation SDK.

It combines:
- A high-performance Rust core (event loop, transport, protocol handling)
- Python bindings built with PyO3
- Async-friendly APIs for sending messages, media, and reacting to events
- Type stubs for better editor support and static checking

## Highlights

- Fast runtime with Rust async internals (Tokio)
- Python async API (`await client.run()`, `await send_message(...)`)
- Event-driven callback registration
- SQLite-backed session/storage integration
- Built-in WhatsApp protobuf access (`tryx.waproto.whatsapp_pb2`)
- Typed Python package (`py.typed` + `.pyi` stubs)

## Repository Layout

- `src/`: Rust PyO3 binding layer and Python-exposed classes
- `python/tryx/`: Python package, dynamic re-exports, type stubs, protobuf module
- `libs/whatsapp-rust/`: WhatsApp Rust stack used by this project
- `examples.py`: basic usage example
- `pyproject.toml`: Python package/build configuration (Maturin)
- `Cargo.toml`: Rust crate dependencies and metadata

## Requirements

- Python 3.8+
- Rust toolchain (stable)
- A Linux/macOS environment is recommended for local builds

## Install (Development)

### 1. Clone and enter project

```bash
git clone <your-repo-url>
cd tryx
```

### 2. Build and install with Maturin

```bash
pip install maturin
maturin develop
```

This builds the Rust extension and installs the Python package in your active environment.

## Quick Start

```python
import asyncio
from tryx.backend import SqliteBackend
from tryx.client import Tryx
from tryx.events import EvMessage
from tryx.waproto.whatsapp_pb2 import Message

backend = SqliteBackend("whatsapp.db")
bot = Tryx(backend)

@bot.on(EvMessage)
async def on_message(*args):
    # Depending on your event wiring, callback payload can be event-only
    # or custom arguments from your dispatcher integration.
    event = args[-1]
    text = event.data.get_text() or "<non-text>"
    chat = event.data.message_info.source.chat
    await bot.get_client().send_message(chat, Message(conversation=f"Echo: {text}"))

async def main() -> None:
    await bot.run()

asyncio.run(main())
```

## Core Python API

### Backend

- `SqliteBackend(path: str)`

### Main client

- `Tryx(backend: BackendBase)`
- `Tryx.on(EventClass)` decorator for handler registration
- `await Tryx.run()`
- `Tryx.run_blocking()`
- `Tryx.get_client()` -> low-level `TryxClient`

### Messaging and media (`TryxClient`)

- `await send_message(to, message)`
- `await send_text(to, text, quoted=None)`
- `await send_photo(to, photo_data, caption, quoted=None)`
- `await download_media(message)`
- `await upload(data, media_type)`
- `await upload_file(path, media_type)`
- `await get_profile_picture(jid, preview)`
- `await is_on_whatsapp([...])`
- `await get_user_info(jid)`

## Typing and IDE Support

Tryx ships with package-level type information:
- `.pyi` stubs under `python/tryx/`
- `py.typed` marker file

Recommended tooling:
- `mypy`
- `pyright`
- VS Code Pylance

## Notes on Event Models

Event payloads are represented as Python classes generated from Rust-side event structs.
Many event classes expose computed properties such as:
- `.data`
- `.node`
- `.action`
- `.proto`

Some fields are lazily materialized for lower runtime overhead.

## Build and Packaging

This project uses:
- **PyO3** for Python bindings
- **Maturin** for wheel/build integration
- **Tokio** for async runtime
- **Prost** for protobuf encode/decode

Build config is defined in:
- `Cargo.toml`
- `pyproject.toml`

## Troubleshooting

### Extension import fails (`No module named tryx._tryx`)

Rebuild/install the extension:

```bash
maturin develop --release
```

### Type checker does not pick up stubs

- Ensure your environment installs the local package
- Confirm `python/tryx/py.typed` is included in the installed distribution
- Restart language server (Pylance/Pyright)

### Runtime backend errors

- Verify SQLite path is writable
- Check logs for backend initialization failures

## License

This repository is distributed under the license in `LICENSE`.
