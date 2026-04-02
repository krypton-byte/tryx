# Tryx

[![CI](https://img.shields.io/github/actions/workflow/status/krypton-byte/tryx/CI.yml?label=CI&style=for-the-badge&logo=githubactions)](https://github.com/krypton-byte/tryx/actions/workflows/CI.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/krypton-byte/tryx/release.yml?label=Release&style=for-the-badge&logo=githubactions)](https://github.com/krypton-byte/tryx/actions/workflows/release.yml)
[![Docs](https://img.shields.io/badge/Docs-Live-0ea5e9?style=for-the-badge&logo=readthedocs&logoColor=white)](http://krypton-byte.tech/tryx/)
[![Python](https://img.shields.io/badge/Python-3.8%2B-3776AB?style=for-the-badge&logo=python&logoColor=white)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/Rust-Stable-000000?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Typed](https://img.shields.io/badge/Typing-PEP%20561-0ea5e9?style=for-the-badge)](https://peps.python.org/pep-0561/)
[![License](https://img.shields.io/github/license/krypton-byte/tryx?style=for-the-badge)](LICENSE)

Tryx is a Rust-powered Python SDK for building WhatsApp automations with an async-first API, strong typing, and production-focused performance.

It combines:

- Rust for protocol and runtime-heavy paths
- PyO3 for Python bindings
- Tokio for async orchestration
- Typed Python package distribution (`.pyi` + `py.typed`)

> Note: This project is an independent developer SDK and is not affiliated with WhatsApp or Meta.

## Why Tryx

- Async-first architecture for event-driven bots
- Python-friendly API with namespace-based clients
- High-performance native core for protocol and transport workloads
- Typed interfaces for better editor support and safer integrations
- Supports both async and blocking runtime styles

## Quick Links

- Documentation: http://krypton-byte.tech/tryx/
- Contributing Guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- Command Automation Example: [examples/command_bot.py](examples/command_bot.py)

## Installation

### Prerequisites

- Python 3.8+
- Rust stable toolchain
- `uv`

### Development Install (Editable)

```bash
uv sync --group dev
uv run maturin develop
```

### Build Wheels

```bash
uv run maturin build --release
```

## Quick Start

```python
import asyncio
from tryx.backend import SqliteBackend
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage
from tryx.waproto.whatsapp_pb2 import Message

backend = SqliteBackend("whatsapp.db")
app = Tryx(backend)

@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = event.data.get_text() or "<non-text>"
    chat = event.data.message_info.source.chat
    await client.send_message(chat, Message(conversation=f"Echo: {text}"))

async def main() -> None:
    await app.run()

if __name__ == "__main__":
    asyncio.run(main())
```

## Feature Overview

- Event-based handlers via `@app.on(...)`
- Runtime client namespaces:
  - `contact`, `chat_actions`, `community`, `newsletter`, `groups`
  - `status`, `chatstate`, `blocking`, `polls`, `presence`, `privacy`, `profile`
- Media upload/download and message sending helpers
- Typed helper utilities under `tryx.helpers`

For complete API coverage, see the docs site and generated API pages.

## Project Layout

- `src/`: Rust core bindings and runtime integration
- `python/tryx/`: Python package surface and type stubs
- `python/tryx/waproto/`: generated protobuf Python modules
- `examples/`: runnable usage examples
- `docs/`: MkDocs sources

## Development Workflow

```bash
# Lint and format check
uv run ruff check .
uv run ruff format --check .

# Validate stubs
uv run python scripts/check_stub_parity.py

# Build docs locally
uv sync --group docs
uv run mkdocs serve
```

## Release Workflow (Maintainers)

- Release is triggered manually via GitHub Actions (`Semantic Release`).
- Version bump is automatic from Conventional Commits:
  - `feat` -> minor
  - `fix` / `perf` -> patch
  - breaking change -> major
- The workflow creates a Git tag (`vX.Y.Z`), creates a GitHub Release, then triggers CI for publish.

## Troubleshooting

### Native Module Import Error

If you see `ModuleNotFoundError: No module named 'tryx._tryx'`:

```bash
uv run maturin develop --release
```

### Client Not Running

Ensure the client runtime is started (`run` or `run_blocking`) before calling runtime client methods.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution and release guidelines.

## License

This project is licensed under the terms in [LICENSE](LICENSE).
