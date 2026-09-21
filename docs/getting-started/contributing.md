# :material-account-group: Contributing

This guide explains how to contribute to Tryx development.

---

## Development Setup

```bash
# Clone
git clone https://github.com/krypton-byte/tryx.git
cd tryx

# Install dependencies
uv sync --group dev

# Build Rust extension
uv run maturin develop

# Verify
uv run python -c "from tryx.client import Tryx; print('OK')"
```

---

## Code Structure

```
tryx/
├── src/                    # Rust source
│   ├── lib.rs              # Module entry point
│   ├── clients/            # Client method implementations
│   ├── events/             # Event dispatcher and types
│   ├── types.rs            # Shared data classes
│   └── backend/            # Storage backend bridge
├── python/tryx/            # Python package
│   ├── *.pyi               # Type stubs (edit these for API changes)
│   └── waproto/            # Protobuf definitions
├── tests/                  # Test suite
└── docs/                   # Documentation
```

---

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

- **Rust changes**: Edit files in `src/`
- **Python API changes**: Edit `.pyi` stubs in `python/tryx/`
- **Documentation**: Edit files in `docs/`

### 3. Build and Test

```bash
# Rebuild Rust extension
uv run maturin develop

# Run tests
uv run pytest

# Type check
uv run mypy your_changes/

# Lint
uv run ruff check .
```

### 4. Commit

Use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Feature
git commit -m "feat: add new group action"

# Bug fix
git commit -m "fix: handle edge case in media download"

# Documentation
git commit -m "docs: improve quickstart guide"
```

### 5. Push and Create PR

```bash
git push origin feature/my-feature
```

---

## Code Style

### Rust

- Follow standard `rustfmt` formatting
- Use `clippy` for linting
- Keep functions focused and well-documented

### Python

- Follow `ruff` formatting (line length 88)
- Use Google-style docstrings in `.pyi` files
- Keep type stubs in sync with Rust implementations

### Documentation

- Use Markdown with Material for MkDocs / Zensical conventions
- Include code examples for all API methods
- Keep explanations concise and actionable

---

## Testing

```bash
# Run all tests
uv run pytest

# Run specific test file
uv run pytest tests/test_types.py

# Run with verbose output
uv run pytest -v
```

---

## Pull Request Guidelines

1. **One feature per PR** — keep changes focused
2. **Include tests** — for new functionality
3. **Update documentation** — if API surface changes
4. **Run CI checks** — before requesting review
5. **Write clear commit messages** — describe what and why

---

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/krypton-byte/tryx/issues)
- **Discussions**: [GitHub Discussions](https://github.com/krypton-byte/tryx/discussions)
