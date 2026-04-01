# Installation

This page sets up a local development environment for the Tryx Python bindings backed by Rust.

!!! note "Recommended shell flow"
	Use `uv` to manage the project environment and dependencies consistently.

## Prerequisites

- Python 3.8+
- Rust toolchain (stable)
- `uv`

## Environment Bootstrap

```bash
uv sync --group dev
```

## Local Development Install

```bash
uv run maturin develop
```

This installs the Rust extension module into your active environment in editable mode.

!!! tip "Fast rebuild loop"
	Re-run `uv run maturin develop` after Rust binding changes to keep Python runtime artifacts in sync.

## Build Wheel

```bash
uv run maturin build --release
```

Typical wheel output appears under `target/wheels/`.

## Verify Installation

```python
from tryx.client import Tryx, TryxClient
from tryx.backend import SqliteBackend

backend = SqliteBackend("whatsapp.db")
bot = Tryx(backend)
client = bot.get_client()
print(type(client).__name__)
```

If output shows `TryxClient`, extension loading is successful.

## Optional Tools

- `uv run mypy ...` or `pyright` for static type checks
- `uv run ruff check .` for linting
- `uv run pytest` for integration test harnesses
- `uv run pre-commit run --all-files` for local gate parity with CI

## Common Install Issues

### Rust compiler not found

Install Rust with `rustup` and reopen your shell.

### Build fails with linker errors

Ensure your platform build tools are installed:

- Linux: `build-essential` and OpenSSL dev headers
- macOS: Xcode command line tools
- Windows: MSVC Build Tools

### ImportError for extension module

Re-run `uv run maturin develop` in the same project environment where you run Python.

## Next Step

- Continue to [Quick Start](quickstart.md)
- Then configure pairing in [Authentication Flow](authentication.md)
