# Installation

This page sets up a local development environment for the Tryx Python bindings backed by Rust.

!!! note "Recommended shell flow"
	Use an isolated virtual environment per project to avoid conflicting native extension builds.

## Prerequisites

- Python 3.8+
- Rust toolchain (stable)
- `pip` and virtual environment tooling

## Environment Bootstrap

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install -U pip
```

## Local Development Install

```bash
python -m venv .venv
source .venv/bin/activate
pip install -U pip maturin
maturin develop
```

This installs the Rust extension module into your active environment in editable mode.

!!! tip "Fast rebuild loop"
	Re-run `maturin develop` after Rust binding changes to keep Python runtime artifacts in sync.

## Build Wheel

```bash
maturin build --release
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

- `mypy` or `pyright` for static type checks
- `ruff` for linting
- `pytest` for integration test harnesses

## Common Install Issues

### Rust compiler not found

Install Rust with `rustup` and reopen your shell.

### Build fails with linker errors

Ensure your platform build tools are installed:

- Linux: `build-essential` and OpenSSL dev headers
- macOS: Xcode command line tools
- Windows: MSVC Build Tools

### ImportError for extension module

Re-run `maturin develop` in the same active virtual environment where you run Python.

## Next Step

- Continue to [Quick Start](quickstart.md)
- Then configure pairing in [Authentication Flow](authentication.md)
