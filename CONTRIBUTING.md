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
4. Use Conventional Commit style commit messages (`feat:`, `fix:`, `perf:`, etc.).
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

- Release is executed manually from GitHub Actions using the `Semantic Release` workflow (`workflow_dispatch`).
- Semantic release evaluates Conventional Commits already present in the repository default branch (from PR merges or direct pushes).
- Semantic release evaluates Conventional Commits already present in the repository default branch (from PR merges or direct pushes).
- If commits qualify (`feat`, `fix`, `perf`, or breaking), version and changelog are updated and a new tag (`vX.Y.Z`) is created.
- If commits do not qualify (for example docs/chore only), release is a no-op and no publish is triggered.
- After a tag is created, GitHub Release notes are generated automatically.
- CI release pipeline is tag-driven (`vX.Y.Z`) to build artifacts and publish to PyPI.
- Workflow dispatch is kept only as internal fallback trigger from `Semantic Release` automation.

## Simple Trigger Guide

Use this rule of thumb for automatic versioning:

- `feat: ...` -> bump **minor** (for example `0.3.1` -> `0.4.0`)
- `fix: ...` or `perf: ...` -> bump **patch** (for example `0.3.1` -> `0.3.2`)
- `feat!: ...` or commit body with `BREAKING CHANGE:` -> bump **major**
- `docs:`, `chore:`, `test:` only -> no release

How to trigger semantic release until publish to PyPI:

1. Push commits to default branch (direct push or merged PR), using Conventional Commit messages.
2. Open GitHub Actions and run `Semantic Release` on the default branch.
3. Choose `release_type`: `auto` (default) for commit-based bump, or `patch`/`minor`/`major` to force bump manually.
4. Workflow evaluates commits and creates tag `vX.Y.Z` when releasable commits exist.
5. Workflow creates the corresponding GitHub Release.
6. CI workflow runs on that tag and publishes artifacts to PyPI.

Manual fallback (if needed):

1. Open Actions tab.
2. Run `Semantic Release` workflow via `workflow_dispatch`.
3. If no releasable commit exists, workflow will report no new release.

Commit validation note:
- Conventional commit message checks run on both `push` and `pull_request` events.

Required repository secrets:
- `PYPI_API_TOKEN`: token used by publish job to upload wheels/sdist to PyPI.
