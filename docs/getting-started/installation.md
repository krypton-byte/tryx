# Installation

## Prerequisites

- Python 3.8+
- Rust toolchain (stable)
- `pip` and virtual environment tooling

## Local Development Install

```bash
python -m venv .venv
source .venv/bin/activate
pip install -U pip maturin
maturin develop
```

This installs the Rust extension module into your active environment in editable mode.

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
