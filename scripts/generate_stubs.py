#!/usr/bin/env python3
"""Auto-generate ``_tryx.pyi`` type stubs from the compiled Rust extension.

Introspects the compiled ``_tryx`` module and produces a comprehensive
``.pyi`` stub file.  Run in CI after a fast build, commit the result
so docs builds never compile Rust.

Usage:
    maturin develop --profile stubs
    PYTHONPATH=python python scripts/generate_stubs.py
"""

from __future__ import annotations

import importlib
import inspect
import sys
import textwrap
import types
from pathlib import Path
from typing import Any, get_type_hints

# ── Configuration ──────────────────────────────────────────────────
PROJECT_ROOT = Path(__file__).resolve().parent.parent
OUTPUT = PROJECT_ROOT / "python" / "tryx" / "_tryx.pyi"
PYTHON_DIR = PROJECT_ROOT / "python"

SUBMODULES = ["backend", "client", "events", "exceptions", "helpers", "types", "wacore"]

INDENT = "    "


def _type_str(t: Any, origin: str = "") -> str:
    """Best-effort annotation → string."""
    if t is inspect.Parameter.empty or t is inspect.Signature.empty:
        return "Any"
    if t is type(None):
        return "None"
    if isinstance(t, type):
        if t.__module__ == "builtins":
            return t.__name__
        return t.__name__
    s = str(t)
    # Strip module prefixes
    for prefix in ("builtins.", "typing.", "_tryx.", origin + "."):
        s = s.replace(prefix, "")
    return s.replace("typing.", "")


def _sig_str(sig: inspect.Signature, origin: str = "") -> str:
    parts = []
    for name, p in sig.parameters.items():
        if name == "self":
            continue
        ann = _type_str(p.annotation, origin) if p.annotation is not inspect.Parameter.empty else "Any"
        if p.default is inspect.Parameter.empty:
            parts.append(f"{name}: {ann}")
        else:
            if p.default is None:
                dv = "None"
            elif isinstance(p.default, (bool, int, float, str)):
                dv = repr(p.default)
            else:
                dv = "..."
            parts.append(f"{name}: {ann} = {dv}")
    return ", ".join(parts)


def _ret_str(func: Any, origin: str = "") -> str:
    try:
        hints = get_type_hints(func)
        if "return" in hints:
            return _type_str(hints["return"], origin)
    except Exception:
        pass
    try:
        sig = inspect.signature(func)
        if sig.return_annotation is not inspect.Signature.empty:
            return _type_str(sig.return_annotation, origin)
    except (ValueError, TypeError):
        pass
    return "Any"


def _is_enum(cls: type) -> bool:
    """Heuristic: all non-dunder attrs are instances of cls."""
    attrs = [a for a in dir(cls) if not a.startswith("_")]
    if not attrs:
        return False
    try:
        return all(isinstance(getattr(cls, a, None), cls) for a in attrs[:5])
    except Exception:
        return False


def _class_stub(cls: type, origin: str = "") -> list[str]:
    name = cls.__name__
    lines: list[str] = []
    doc = (cls.__doc__ or "").strip().split("\n")[0]

    if _is_enum(cls):
        # ── Enum-like ──
        attrs = [a for a in dir(cls) if not a.startswith("_")]
        variants = [a for a in attrs if isinstance(getattr(cls, a, None), cls)]
        if doc:
            lines.append(f'"""{doc}"""')
        lines.append(f"class {name}:")
        for v in variants:
            lines.append(f"{INDENT}{v}: ClassVar[{name}]")
        return lines

    # ── Regular class ──
    data_attrs: list[tuple[str, str]] = []
    methods: list[str] = []
    properties: list[tuple[str, str]] = []

    for attr_name in sorted(dir(cls)):
        if attr_name.startswith("_") and attr_name != "__init__":
            continue
        try:
            attr = getattr(cls, attr_name)
        except Exception:
            continue

        if attr_name == "__init__":
            try:
                sig = inspect.signature(cls)
                ps = _sig_str(sig, origin)
                methods.append(f"{INDENT}def __init__(self, {ps}) -> None: ...")
            except (ValueError, TypeError):
                methods.append(f"{INDENT}def __init__(self, **kwargs: Any) -> None: ...")
            continue

        if isinstance(attr, cls):
            continue  # enum variant handled above

        if isinstance(attr, property):
            ret = _ret_str(attr.fget, origin) if attr.fget else "Any"
            properties.append((attr_name, ret))
            continue

        if callable(attr):
            try:
                sig = inspect.signature(attr)
                ps = _sig_str(sig, origin)
                ret = _ret_str(attr, origin)
                methods.append(f"{INDENT}def {attr_name}({ps}) -> {ret}: ...")
            except (ValueError, TypeError):
                methods.append(f"{INDENT}def {attr_name}(*args: Any, **kwargs: Any) -> Any: ...")
        else:
            # Data attribute
            type_s = _type_str(type(attr), origin)
            data_attrs.append((attr_name, type_s))

    if doc:
        lines.append(f'"""{doc}"""')
    lines.append(f"class {name}:")

    if not (data_attrs or methods or properties):
        lines.append(f"{INDENT}pass")

    for da_name, da_type in data_attrs:
        lines.append(f"{INDENT}{da_name}: {da_type}")
    for prop_name, prop_ret in properties:
        lines.append(f"{INDENT}@property")
        lines.append(f"{INDENT}def {prop_name}(self) -> {prop_ret}: ...")
    for m in methods:
        lines.append(m)

    return lines


def _module_stub(mod: types.ModuleType, mod_name: str) -> list[str]:
    lines: list[str] = [f"# ── _tryx.{mod_name} ──", ""]
    seen: set[str] = set()

    for attr_name in sorted(dir(mod)):
        if attr_name.startswith("_") or attr_name in seen:
            continue
        try:
            obj = getattr(mod, attr_name)
        except Exception:
            continue
        seen.add(attr_name)

        if isinstance(obj, type):
            lines.extend(_class_stub(obj, mod_name))
            lines.append("")
        elif callable(obj) and not isinstance(obj, type):
            try:
                sig = inspect.signature(obj)
                ps = _sig_str(sig, mod_name)
                ret = _ret_str(obj, mod_name)
                doc = (obj.__doc__ or "").strip().split("\n")[0]
                if doc:
                    lines.append(f'"""{doc}"""')
                lines.append(f"def {attr_name}({ps}) -> {ret}: ...")
                lines.append("")
            except (ValueError, TypeError):
                pass

    return lines


def main() -> int:
    sys.path.insert(0, str(PYTHON_DIR))

    try:
        from tryx import _tryx  # type: ignore
    except ImportError as e:
        print(f"❌ Cannot import _tryx: {e}")
        print("   Build first: maturin develop --profile stubs")
        return 1

    header = '''\
"""
Auto-generated type stubs for the compiled Rust extension ``_tryx``.

Generated by ``scripts/generate_stubs.py`` — do not edit manually.
Regenerate after Rust source changes:

    maturin develop --profile stubs
    PYTHONPATH=python python scripts/generate_stubs.py
"""

from __future__ import annotations
from typing import Any, ClassVar, overload
'''
    parts: list[str] = [header]

    for submod_name in SUBMODULES:
        try:
            submod = getattr(_tryx, submod_name, None)
            if submod is None:
                submod = importlib.import_module(f"tryx._tryx.{submod_name}")
        except ImportError:
            print(f"⚠  Skipping _tryx.{submod_name}")
            continue

        print(f"✓  _tryx.{submod_name}")
        parts.extend(_module_stub(submod, submod_name))
        parts.append("")

    OUTPUT.write_text("\n".join(parts))
    size_kb = OUTPUT.stat().st_size / 1024
    n_classes = sum(1 for l in parts if l.strip().startswith("class "))
    n_methods = sum(1 for l in parts if "def " in l and "(self" in l)
    print(f"\n✅ {OUTPUT.name}: {size_kb:.1f} KB, {n_classes} classes, {n_methods} methods")
    return 0


if __name__ == "__main__":
    sys.exit(main())
