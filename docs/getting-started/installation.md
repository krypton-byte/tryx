# Installation

This guide sets up a working development environment for building with Tryx.

!!! tip "One command setup"
    ```bash
    uv sync --group dev && uv run maturin develop
    ```

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Python** | 3.8+ | Runtime |
| **Rust** | stable | Native extension compilation |
| **uv** | latest | Package management |
| **OpenSSL** | 1.1+ | TLS for WebSocket connections |

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Install uv

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Quick Setup

```bash
# Clone the repository
git clone https://github.com/krypton-byte/tryx.git
cd tryx

# Install all dependencies
uv sync --group dev

# Build the Rust extension into your environment
uv run maturin develop
```

!!! success "Verify installation"
    ```bash
    uv run python -c "from tryx.client import Tryx; print('Tryx loaded successfully')"
    ```
    If this prints `Tryx loaded successfully`, you're ready to go.

## Build Options

### Development Build (fast compilation)

```bash
uv run maturin develop
```

This installs the extension module in editable mode. Re-run after Rust source changes.

### Release Build (optimized binary)

```bash
uv run maturin build --release
```

The wheel is output to `target/wheels/`.

### With Specific Features

```bash
# Verbose build for debugging
uv run maturin develop -v

# Release with debug symbols
uv run maturin develop --release --cargo-extra-args="--profile dev"
```

## Project Layout

```
tryx/
├── Cargo.toml              # Rust dependencies and build config
├── pyproject.toml          # Python dependencies and tool config
├── src/                    # Rust source code
│   ├── lib.rs              # Module entry point
│   ├── clients/            # Client implementations
│   ├── events/             # Event dispatcher
│   └── types.rs            # Shared data types
├── python/tryx/            # Python package
│   ├── __init__.py         # Re-exports
│   ├── *.pyi               # Type stubs for IDE support
│   └── waproto/            # Protobuf definitions
├── tests/                  # Test suite
└── examples/               # Usage examples
```

## Common Issues

### Rust compiler not found

```bash
# Install Rust toolchain
rustup default stable
```

### Build fails with linker errors

=== "Linux"

    ```bash
    sudo apt install build-essential libssl-dev pkg-config
    ```

=== "macOS"

    ```bash
    xcode-select --install
    ```

=== "Windows"

    Install [MSVC Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

### ImportError for extension module

Make sure you're using the same Python environment where `maturin develop` was run:

```bash
# Check which Python your venv uses
which python

# Rebuild if needed
uv run maturin develop
```

### Protobuf version mismatch

Tryx uses protobuf 5.28+ for code generation. If you see version warnings:

```bash
uv add "protobuf>=5.28.3,<7"
uv run maturin develop
```

## Optional Tools

| Tool | Install | Purpose |
|------|---------|---------|
| **mypy** | `uv add mypy` | Static type checking |
| **pyright** | `uv add pyright` | Type checking alternative |
| **ruff** | `uv add ruff` | Linting and formatting |
| **pytest** | `uv add pytest` | Test runner |

```bash
# Type check
uv run mypy your_project/

# Lint
uv run ruff check .

# Format
uv run ruff format .

# Test
uv run pytest
```

## Next Step

→ [Quick Start](quickstart.md) — build your first bot in 5 minutes
