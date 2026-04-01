# Changelog Policy

Tryx changelog is generated automatically with `python-semantic-release`.

Generated file: `CHANGELOG.md`

Do not edit generated release sections manually.

## Source of Truth

- Conventional Commit messages from merged history on `main`
- Semantic version tags in format `vX.Y.Z`

## Version Bump Rules

- `feat` -> minor bump
- `fix` and `perf` -> patch bump
- `!` or `BREAKING CHANGE:` footer -> major bump

Commits outside release types (`docs`, `chore`, `ci`, etc.) are still tracked in git history but do not necessarily trigger a release.

## Minimum Entry Rules

Each release entry should include:

1. user-facing summary
2. affected modules (`client`, `events`, `types`, etc.)
3. migration notes if behavior changed
4. compatibility impact (if any)

The release workflow produces these sections from commit metadata.

## API Surface Notes

Whenever a PyO3 class is added or removed:

- update Rust `add_class` registration
- update corresponding `.pyi`
- update API reference docs
- include change note in release entry
