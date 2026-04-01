# Contributing to Tryx

Thanks for contributing to Tryx.

## Development Setup (uv-first)

1. Install `uv` and Rust toolchain.
2. Sync local dependencies:

```bash
uv sync --group dev --group docs
```

3. Build native extension in editable mode:

```bash
uv run maturin develop
```

4. Install local hooks:

```bash
uv run pre-commit install --hook-type pre-commit --hook-type commit-msg
```

## Code Quality Rules

Run these before pushing:

```bash
uv run ruff check .
uv run ruff format --check .
uv run python scripts/check_stub_parity.py
```

## Commit Message Rules (Conventional Commits)

Use format:

```text
type(scope): summary
type(scope)!: summary
```

Scope is optional.

Allowed `type` values:
- `feat`
- `fix`
- `perf`
- `refactor`
- `docs`
- `test`
- `build`
- `ci`
- `chore`
- `style`

Versioning rules for semantic release:
- `feat` -> minor release
- `fix` and `perf` -> patch release
- `!` or `BREAKING CHANGE:` footer -> major release

Examples:
- `feat(client): add media retry metadata`
- `fix(events): avoid panic on missing participant`
- `feat(api)!: rename send_file to upload_file`

## Pull Request Process

1. Create a feature branch from `main`.
2. Keep PR scope focused and small.
3. Ensure CI is green.
4. Use a Conventional Commit style PR title.
5. Fill the PR template completely.

Recommended merge strategy: squash merge.

## Issue Reporting

Use issue templates:
- Bug report for runtime/behavior bugs
- Feature request for API or workflow enhancements

Include:
- reproduction steps
- expected vs actual behavior
- environment details

## Release Flow Overview

- Release automation runs when a PR from `dev` into `main` is merged.
- Semantic release evaluates Conventional Commits from merged changes.
- If commits qualify (`feat`, `fix`, `perf`, or breaking), version and changelog are updated and a new tag (`vX.Y.Z`) is created.
- If commits do not qualify (for example docs/chore only), release is a no-op and no publish is triggered.
- New `vX.Y.Z` tags trigger the CI workflow that builds multi-platform wheels and publishes to PyPI.

Required repository secrets:
- `RELEASE_PUSH_TOKEN`: personal access token used by semantic-release workflow to push release commit and tags.
- `PYPI_API_TOKEN`: token used by publish job to upload wheels/sdist to PyPI.
