# Contributing Guide

This page describes how to contribute to Tryx using the project standards for tooling, commits, and pull requests.

## Local Setup

```bash
uv sync --group dev --group docs
uv run maturin develop
uv run pre-commit install --hook-type pre-commit --hook-type commit-msg
```

## Required Checks

```bash
uv run --no-project --with ruff==0.11.4 ruff check .
uv run --no-project --with ruff==0.11.4 ruff format --check .
uv run python scripts/check_stub_parity.py
```

## Commit Message Standard

Tryx uses Conventional Commits.

Pattern:

```text
type(scope): summary
type(scope)!: summary
```

Scope is optional.

Allowed `type`:
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

Release impact:
- `feat` -> minor bump
- `fix` / `perf` -> patch bump
- `!` or `BREAKING CHANGE:` -> major bump

Examples:
- `feat(groups): add participant count helper`
- `fix(profile): validate image bytes before upload`
- `feat(status)!: change default privacy behavior`

## Pull Requests

1. Use the PR template.
2. Keep title in Conventional Commit format.
3. Link relevant issue(s).
4. Add tests/docs updates when behavior changes.
5. Ensure CI is green before requesting review.

## Issues

Use issue templates for:
- Bug reports
- Feature requests

High-quality issues include:
- clear reproduction steps
- expected and actual results
- logs or traceback snippets
- OS / Python / Tryx version details

## Additional Recommendations

- Prefer squash merge for consistent release history.
- Keep PR size under review-friendly scope.
- Update docs and stubs together with API changes.
