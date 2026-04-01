#!/usr/bin/env python3
"""Compare runtime-exported classes against .pyi class declarations.

This check helps keep PyO3 class exports and type stubs in sync.
"""

from __future__ import annotations

import importlib
import re
from pathlib import Path

MODULES = [
    "tryx.client",
    "tryx.events",
    "tryx.backend",
    "tryx.exceptions",
    "tryx.types",
    "tryx.wacore",
    "tryx.helpers",
]


CLASS_PATTERN = re.compile(r"^class\s+([A-Za-z_][A-Za-z0-9_]*)")


def classes_from_stub(stub_path: Path) -> set[str]:
    names: set[str] = set()
    for line in stub_path.read_text(encoding="utf-8").splitlines():
        match = CLASS_PATTERN.match(line.strip())
        if match:
            names.add(match.group(1))
    return names


def classes_from_runtime(module_name: str) -> set[str]:
    mod = importlib.import_module(module_name)
    names: set[str] = set()
    for name in dir(mod):
        obj = getattr(mod, name)
        if isinstance(obj, type):
            names.add(name)
    return names


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    stub_root = repo_root / "python" / "tryx"

    has_diff = False
    for module_name in MODULES:
        short = module_name.split(".")[-1]
        stub_path = stub_root / f"{short}.pyi"
        if not stub_path.exists():
            print(f"[WARN] Missing stub file for module {module_name}: {stub_path}")
            has_diff = True
            continue

        stub_classes = classes_from_stub(stub_path)
        runtime_classes = classes_from_runtime(module_name)

        missing_in_stub = sorted(runtime_classes - stub_classes)
        missing_in_runtime = sorted(stub_classes - runtime_classes)

        if missing_in_stub or missing_in_runtime:
            has_diff = True
            print(f"\n[DIFF] {module_name}")
            if missing_in_stub:
                print("  Runtime only:")
                for name in missing_in_stub:
                    print(f"    - {name}")
            if missing_in_runtime:
                print("  Stub only:")
                for name in missing_in_runtime:
                    print(f"    - {name}")
        else:
            print(f"[OK] {module_name} classes are in parity")

    if has_diff:
        print("\nParity check failed.")
        return 1

    print("\nParity check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
