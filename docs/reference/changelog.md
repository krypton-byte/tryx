# Changelog Policy

This page defines how Tryx changes should be documented.

## Categories

- Added
- Changed
- Fixed
- Deprecated
- Removed
- Security

## Minimum Entry Rules

Each release entry should include:

1. user-facing summary
2. affected modules (`client`, `events`, `types`, etc.)
3. migration notes if behavior changed
4. compatibility impact (if any)

## API Surface Notes

Whenever a PyO3 class is added or removed:

- update Rust `add_class` registration
- update corresponding `.pyi`
- update API reference docs
- include change note in release entry
