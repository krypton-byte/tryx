#!/usr/bin/env python3
"""Validate Conventional Commit messages for local hooks and CI."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

ALLOWED_TYPES = (
    "build",
    "chore",
    "ci",
    "docs",
    "feat",
    "fix",
    "perf",
    "refactor",
    "revert",
    "style",
    "test",
)

HEADER_RE = re.compile(
    rf"^(?P<type>{'|'.join(ALLOWED_TYPES)})"
    r"(\([a-z0-9][a-z0-9._/-]*\))?"
    r"(?P<breaking>!)?: .+"
)


def is_valid_subject(subject: str) -> bool:
    subject = subject.strip()
    if not subject:
        return True
    if subject.startswith("Merge "):
        return True
    return bool(HEADER_RE.match(subject))


def validate_commit_message_file(path: Path) -> int:
    if not path.exists():
        print(f"error: commit message file not found: {path}")
        return 2

    first_line = path.read_text(encoding="utf-8").splitlines()
    subject = first_line[0] if first_line else ""

    if is_valid_subject(subject):
        return 0

    print("error: invalid Conventional Commit header")
    print(f"  got: {subject}")
    print("  expected: type(optional-scope)!: short summary")
    print("  example: feat(api)!: remove deprecated send_media alias")
    return 1


def validate_commit_range(revision_range: str) -> int:
    cmd = ["git", "log", "--no-merges", "--format=%h%x09%s", revision_range]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        print(proc.stderr.strip() or "error: failed to read git log range")
        return proc.returncode

    invalid: list[tuple[str, str]] = []
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        commit_hash, _, subject = line.partition("\t")
        if not is_valid_subject(subject):
            invalid.append((commit_hash, subject))

    if not invalid:
        return 0

    print("error: invalid commit subjects found")
    for commit_hash, subject in invalid:
        print(f"  - {commit_hash}: {subject}")
    print("expected format: type(optional-scope)!: summary")
    print("example: feat(auth): support pairing QR refresh")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "message_file",
        nargs="?",
        help="path to commit message file (used by pre-commit commit-msg hook)",
    )
    parser.add_argument(
        "--range",
        dest="revision_range",
        help="git revision range to validate, e.g. abc123..def456",
    )
    args = parser.parse_args()

    if bool(args.message_file) == bool(args.revision_range):
        parser.error("provide either message_file or --range")

    if args.revision_range:
        return validate_commit_range(args.revision_range)

    return validate_commit_message_file(Path(args.message_file))


if __name__ == "__main__":
    raise SystemExit(main())
