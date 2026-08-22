#!/usr/bin/env python3
"""Auto-generate ``_tryx.pyi`` type stubs from Rust source files.

This script parses the Rust source code directly to extract accurate type
information for PyO3 classes, methods, getters, and setters.  Python-level
introspection via ``inspect`` cannot handle PyO3 extension types (they appear
as ``getset_descriptor`` and lack proper ``__init__`` signatures), so we read
the source instead.

Usage:
    maturin develop --profile stubs
    PYTHONPATH=python python scripts/generate_stubs.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Any

# ── Configuration ──────────────────────────────────────────────────
PROJECT_ROOT = Path(__file__).resolve().parent.parent
OUTPUT = PROJECT_ROOT / "python" / "tryx" / "_tryx.pyi"
SRC_DIR = PROJECT_ROOT / "src"

INDENT = "    "

# ── Rust → Python type mapping ─────────────────────────────────────
RUST_TO_PYTHON: dict[str, str] = {
    # Primitives
    "String": "str",
    "bool": "bool",
    "i8": "int",
    "i16": "int",
    "i32": "int",
    "i64": "int",
    "u8": "int",
    "u16": "int",
    "u32": "int",
    "u64": "int",
    "f32": "float",
    "f64": "float",
    "usize": "int",
    # Bytes
    "Vec<u8>": "bytes",
    "&[u8]": "bytes",
    # PyO3 types
    "PyDateTime": "datetime",
    # Unit type
    "()": "None",
    # None-able
    "Option<String>": "str | None",
    "Option<bool>": "bool | None",
    "Option<i32>": "int | None",
    "Option<i64>": "int | None",
    "Option<u32>": "int | None",
    "Option<u64>": "int | None",
}

# ── PyO3 known type aliases ────────────────────────────────────────
# Maps Rust PyO3 type names to their Python equivalents
PYO3_TYPE_MAP: dict[str, str] = {
    "Py<PyAny>": "Any",
    "Py<PyDict>": "dict[str, Any]",
    "Bound<'py, PyAny>": "Any",
}

# Additional Rust → Python type mappings for PyO3/chrono types
RUST_TO_PYTHON_EXTRA: dict[str, str] = {
    "PyDateTime": "datetime",
    "PyObject": "Any",
    "chrono::DateTime<chrono::Utc>": "datetime",
    "String": "str",
}

# ── Module → file mapping ──────────────────────────────────────────
# Maps submodule names to their Rust source directories
MODULE_SOURCES: dict[str, list[Path]] = {
    "types": [SRC_DIR / "types.rs"],
    "backend": [
        SRC_DIR / "backend" / "mod.rs",
        SRC_DIR / "backend" / "store.rs",
        SRC_DIR / "backend" / "store_types.rs",
    ],
    "client": [
        SRC_DIR / "clients" / "tryx.rs",
        SRC_DIR / "clients" / "tryx_client.rs",
        SRC_DIR / "clients" / "contacts.rs",
        SRC_DIR / "clients" / "groups.rs",
        SRC_DIR / "clients" / "newsletter.rs",
        SRC_DIR / "clients" / "polls.rs",
        SRC_DIR / "clients" / "status.rs",
        SRC_DIR / "clients" / "presence.rs",
        SRC_DIR / "clients" / "privacy.rs",
        SRC_DIR / "clients" / "profile.rs",
        SRC_DIR / "clients" / "blocking.rs",
        SRC_DIR / "clients" / "chatstate.rs",
        SRC_DIR / "clients" / "comments.rs",
        SRC_DIR / "clients" / "community.rs",
        SRC_DIR / "clients" / "events.rs",
        SRC_DIR / "clients" / "labels.rs",
        SRC_DIR / "clients" / "advanced.rs",
        SRC_DIR / "clients" / "chat_actions.rs",
        SRC_DIR / "clients" / "voip.rs",
    ],
    "events": [
        SRC_DIR / "events" / "types.rs",
        SRC_DIR / "events" / "types" / "message_and_updates.rs",
        SRC_DIR / "events" / "types" / "profile_sync.rs",
        SRC_DIR / "events" / "types" / "client.rs",
        SRC_DIR / "events" / "dispatcher.rs",
    ],
    "exceptions": [SRC_DIR / "exceptions" / "exceptions.rs"],
    "helpers": [
        SRC_DIR / "helpers" / "blocking.rs",
        SRC_DIR / "helpers" / "chatstate.rs",
        SRC_DIR / "helpers" / "groups.rs",
        SRC_DIR / "helpers" / "newsletter.rs",
        SRC_DIR / "helpers" / "polls.rs",
        SRC_DIR / "helpers" / "presence.rs",
        SRC_DIR / "helpers" / "status.rs",
    ],
    "wacore": [
        SRC_DIR / "wacore" / "node.rs",
        SRC_DIR / "wacore" / "stanza.rs",
        SRC_DIR / "wacore" / "download.rs",
        SRC_DIR / "wacore" / "iq" / "usync.rs",
        SRC_DIR / "wacore" / "iq" / "groups.rs",
        SRC_DIR / "wacore" / "iq" / "community.rs",
        SRC_DIR / "wacore" / "iq" / "newsletter.rs",
        SRC_DIR / "wacore" / "iq" / "polls.rs",
        SRC_DIR / "wacore" / "iq" / "presence.rs",
        SRC_DIR / "wacore" / "iq" / "privacy.rs",
        SRC_DIR / "wacore" / "iq" / "status.rs",
        SRC_DIR / "wacore" / "iq" / "blocking.rs",
    ],
}


# ══════════════════════════════════════════════════════════════════
# Rust Source Parser
# ══════════════════════════════════════════════════════════════════

class RustClass:
    """Represents a parsed PyO3 class from Rust source."""
    def __init__(self, name: str, doc: str = ""):
        self.name = name
        self.doc = doc
        self.fields: list[tuple[str, str, bool]] = []  # (name, type, has_get_set)
        self.getters: list[tuple[str, str]] = []  # (name, return_type)
        self.setters: list[tuple[str, str]] = []  # (name, param_type)
        self.methods: list[tuple[str, str, list[tuple[str, str, str | None]]]] = (
            []  # (name, return_type, [(param_name, param_type, default)])
        )
        self.is_enum = False
        self.enum_variants: list[str] = []
        self.static_methods: list[tuple[str, str, list[tuple[str, str, str | None]]]] = []


def rust_type_to_python(rust_type: str) -> str:
    """Convert a Rust type string to its Python equivalent."""
    t = rust_type.strip()

    # Remove lifetime annotations early
    t = re.sub(r"<'[^>]+>", "", t)

    # Strip crate/module prefixes
    t = re.sub(r"^(?:crate|pyo3|wacore|whatsapp_rust|waproto)::[\w:]*::", "", t)
    t = re.sub(r"^pyo3::", "", t)

    # Handle Option<T>
    opt_match = re.match(r"Option<(.+)>", t)
    if opt_match:
        inner = rust_type_to_python(opt_match.group(1))
        return f"{inner} | None"

    # Handle Vec<T>
    vec_match = re.match(r"Vec<(.+)>", t)
    if vec_match:
        inner = rust_type_to_python(vec_match.group(1))
        if inner == "u8":
            return "bytes"
        return f"list[{inner}]"

    # Handle &str
    if t == "&str":
        return "str"

    # Handle &T
    if t.startswith("&"):
        return rust_type_to_python(t[1:])

    # Handle slices
    if t.startswith("&[") and t.endswith("]"):
        inner = rust_type_to_python(t[2:-1])
        if inner == "u8":
            return "bytes"
        return f"list[{inner}]"

    # Handle tuple (Python: tuple[...])
    if t.startswith("(") and t.endswith(")"):
        inner_str = t[1:-1].strip()
        if inner_str:
            parts = _split_type_args(inner_str)
            inner = ", ".join(rust_type_to_python(p) for p in parts)
            return f"tuple[{inner}]"
        return "tuple[()]"

    # Handle Py<T> → T
    py_match = re.match(r"Py<(.+)>", t)
    if py_match:
        inner = py_match.group(1).strip()
        if inner == "PyAny":
            return "Any"
        if inner == "PyDict":
            return "dict[str, Any]"
        return rust_type_to_python(inner)

    # Handle Bound<'py, T> → T
    bound_match = re.match(r"Bound<'[^>]*,\s*(.+)>", t)
    if bound_match:
        inner = bound_match.group(1).strip()
        if inner == "PyAny":
            return "Any"
        return rust_type_to_python(inner)

    # Handle PyResult<T> → T
    if t == "PyResult<()>":
        return "None"
    if t.startswith("PyResult<") and t.endswith(">"):
        inner = t[len("PyResult<"):-1]
        return rust_type_to_python(inner)

    # Handle Result<T, E> → T
    if t.startswith("Result<") and t.endswith(">"):
        inner = t[len("Result<"):-1]
        parts = _split_type_args(inner)
        if parts:
            return rust_type_to_python(parts[0])
        return "Any"

    # Simple lookup
    if t in RUST_TO_PYTHON:
        return RUST_TO_PYTHON[t]
    if t in RUST_TO_PYTHON_EXTRA:
        return RUST_TO_PYTHON_EXTRA[t]

    # Remove remaining lifetime annotations
    t = re.sub(r"<'[^>]+>", "", t)

    # Handle Py<Self> → Self (for static methods)
    if t == "Py<Self>":
        return "Self"

    # 'static str → str
    if t == "'static str":
        return "str"

    # Clean up any remaining lifetime prefixes
    t = re.sub(r"^'[a-z]+\s+", "", t)

    return t


def _split_type_args(s: str) -> list[str]:
    """Split a comma-separated type string, respecting angle brackets and parens."""
    parts = []
    angle_depth = 0
    paren_depth = 0
    current = ""
    for ch in s:
        if ch == "<":
            angle_depth += 1
            current += ch
        elif ch == ">":
            angle_depth = max(0, angle_depth - 1)
            current += ch
        elif ch == "(":
            paren_depth += 1
            current += ch
        elif ch == ")":
            paren_depth = max(0, paren_depth - 1)
            current += ch
        elif ch == "," and angle_depth == 0 and paren_depth == 0:
            parts.append(current.strip())
            current = ""
        else:
            current += ch
    if current.strip():
        parts.append(current.strip())
    return parts


def _extract_return_type(sig: str) -> str:
    """Extract return type from a Rust method signature."""
    # Handle -> ReturnType by collecting until we hit an unbalanced '{'
    arrow_idx = sig.find("->")
    if arrow_idx == -1:
        return "()"
    rest = sig[arrow_idx + 2:].strip()
    # Collect the return type, respecting angle brackets and parens
    angle_depth = 0
    paren_depth = 0
    result = ""
    for ch in rest:
        if ch == "<":
            angle_depth += 1
            result += ch
        elif ch == ">":
            if angle_depth > 0:
                angle_depth -= 1
            result += ch
        elif ch == "(":
            paren_depth += 1
            result += ch
        elif ch == ")":
            if paren_depth > 0:
                paren_depth -= 1
            result += ch
        elif ch == "{" and angle_depth == 0 and paren_depth == 0:
            break
        elif ch in ("\n", "\r"):
            break
        else:
            result += ch
    ret = result.strip()
    return ret if ret else "()"


def _extract_params(body: str, sig_line: str) -> list[tuple[str, str, str | None]]:
    """Extract parameter list from a method signature."""
    params = []
    # Combine multi-line signatures
    full_sig = sig_line
    # Try to find the full signature including return type
    # Look for fn name(params) -> RetType { or fn name(params) {

    # Extract parameters between parentheses
    paren_match = re.search(r"\(([^)]*)\)", full_sig)
    if not paren_match:
        return params

    param_str = paren_match.group(1)
    parts = _split_type_args(param_str)

    for part in parts:
        part = part.strip()
        if not part or part == "&self" or part == "self" or part == "&mut self":
            continue
        # Skip Python-specific params
        if part.startswith("py:") or part.startswith("_py:"):
            continue
        # Parse name: type = default
        # Handle name: Type = default
        eq_match = re.search(r"\s*=\s*(.+)$", part)
        default = None
        if eq_match:
            default = eq_match.group(1).strip()
            part = part[:eq_match.start()].strip()

        colon_match = re.search(r":\s*(.+)$", part)
        if colon_match:
            name = part[:colon_match.start()].strip()
            typ = colon_match.group(1).strip()
            # Clean up lifetimes
            typ = re.sub(r"<'[^>]+>", "", typ)
            params.append((name, typ, default))
        else:
            # No type annotation, skip
            pass

    return params


def parse_pyclass_enum(content: str) -> list[RustClass]:
    """Parse `#[pyclass] enum` blocks."""
    classes = []
    # Match: #[pyclass]\npub enum Name { ... }
    pattern = re.compile(
        r"#\[pyclass[^\]]*\]\s*(?:#\[derive[^\]]*\]\s*)?"
        r"pub\s+enum\s+(\w+)\s*\{([^}]*)\}",
        re.MULTILINE | re.DOTALL,
    )
    for m in pattern.finditer(content):
        name = m.group(1)
        body = m.group(2)
        cls = RustClass(name)
        cls.is_enum = True
        # Extract variant names
        for line in body.split("\n"):
            line = line.strip().rstrip(",")
            if line and not line.startswith("//") and line[0].isupper():
                variant = line.split("(")[0].split("{")[0].split("=")[0].strip()
                if variant:
                    cls.enum_variants.append(variant)
        classes.append(cls)
    return classes


def parse_pyclass_struct(content: str) -> list[RustClass]:
    """Parse `#[pyclass] struct` blocks and extract field-level #[pyo3(get, set)]."""
    classes = []
    # Match: #[pyclass]\npub struct Name { fields }
    pattern = re.compile(
        r"#\[pyclass[^\]]*\]\s*(?:#\[derive[^\]]*\]\s*)?"
        r"(?:pub\s+)?struct\s+(\w+)\s*\{([^}]*)\}",
        re.MULTILINE | re.DOTALL,
    )
    for m in pattern.finditer(content):
        name = m.group(1)
        body = m.group(2)
        cls = RustClass(name)
        # Extract fields with #[pyo3(get)] or #[pyo3(get, set)]
        field_pattern = re.compile(
            r"#\[pyo3\(([^)]*)\)\]\s*(?:pub\s+)?(\w+)\s*:\s*([^,\n]+)",
            re.MULTILINE,
        )
        for fm in field_pattern.finditer(body):
            attrs = fm.group(1)
            field_name = fm.group(2)
            field_type = fm.group(3).strip()
            if "get" in attrs:
                has_set = "set" in attrs
                py_type = rust_type_to_python(field_type)
                cls.fields.append((field_name, py_type, has_set))
        classes.append(cls)
    return classes


def parse_pymethods(content: str) -> dict[str, RustClass]:
    """Parse all #[pymethods] impl blocks and return a dict of class_name → RustClass."""
    result: dict[str, RustClass] = {}

    # Find all impl blocks
    # Pattern: #[pymethods]\nimpl ClassName { ... }
    impl_pattern = re.compile(
        r"#\[pymethods\]\s*impl\s+(\w+)\s*\{(.*?)\n\}",
        re.MULTILINE | re.DOTALL,
    )

    for m in impl_pattern.finditer(content):
        class_name = m.group(1)
        body = m.group(2)

        if class_name not in result:
            result[class_name] = RustClass(class_name)
        cls = result[class_name]

        # Parse methods
        _parse_methods_from_body(cls, body)

    return result


def _parse_methods_from_body(cls: RustClass, body: str):
    """Parse individual methods from an impl block body."""
    lines = body.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i].strip()

        # Skip comments and empty lines
        if not line or line.startswith("//"):
            i += 1
            continue

        # Check for #[getter]
        is_getter = False
        is_setter = False
        is_new = False
        is_static = False
        is_repr = False

        while line.startswith("#["):
            if "#[getter]" in line:
                is_getter = True
            elif "#[setter]" in line:
                is_setter = True
            elif "#[new]" in line:
                is_new = True
            elif "#[staticmethod]" in line:
                is_static = True
            elif "#[pyo3(signature" in line:
                # Extract signature override
                pass
            i += 1
            if i < len(lines):
                line = lines[i].strip()
            else:
                break

        # Skip non-fn lines
        if not line.startswith("fn ") and not line.startswith("pub fn "):
            i += 1
            continue

        # Extract method name
        fn_match = re.match(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)", line)
        if not fn_match:
            i += 1
            continue

        method_name = fn_match.group(1)
        if method_name.startswith("__") and method_name != "__init__":
            i += 1
            continue
        if method_name == "__repr__":
            i += 1
            continue

        # Skip internal Rust-only methods (not exposed to Python)
        # Check if it has Python-visible parameters
        is_special = method_name in ("__init__",)

        # Extract full signature (may span multiple lines)
        # Collect until we find the opening '{' of the method body
        sig_text = line
        # If the line already has a '{', the signature is complete
        # Otherwise, keep reading lines until we find it
        if '{' not in sig_text:
            while i + 1 < len(lines):
                i += 1
                next_line = lines[i].strip()
                sig_text += ' ' + next_line
                if '{' in next_line:
                    break

        # Extract parameters and return type
        params = _extract_params(body, sig_text)
        ret_type = _extract_return_type(sig_text)

        # Convert Rust types to Python
        py_ret = rust_type_to_python(ret_type)
        py_params = [(n, rust_type_to_python(t), d) for n, t, d in params]

        if is_new:
            cls.methods.insert(0, ("__init__", py_ret, py_params))
        elif is_getter:
            cls.getters.append((method_name, py_ret))
        elif is_setter:
            cls.setters.append((method_name, py_params[0][1] if py_params else "Any"))
        elif is_static:
            cls.static_methods.append((method_name, py_ret, py_params))
        else:
            cls.methods.append((method_name, py_ret, py_params))

        i += 1


def parse_rust_sources(module_name: str) -> list[RustClass]:
    """Parse all Rust source files for a given module."""
    files = MODULE_SOURCES.get(module_name, [])
    all_classes: dict[str, RustClass] = {}

    for filepath in files:
        if not filepath.exists():
            continue
        content = filepath.read_text(encoding="utf-8")

        # Parse enums first
        for cls in parse_pyclass_enum(content):
            if cls.name not in all_classes:
                all_classes[cls.name] = cls

        # Parse struct fields
        for cls in parse_pyclass_struct(content):
            if cls.name in all_classes:
                # Merge fields into existing class
                existing = all_classes[cls.name]
                for f in cls.fields:
                    if not any(ef[0] == f[0] for ef in existing.fields):
                        existing.fields.append(f)
            else:
                all_classes[cls.name] = cls

        # Parse methods
        methods = parse_pymethods(content)
        for class_name, cls in methods.items():
            if class_name in all_classes:
                existing = all_classes[class_name]
                # Merge methods
                if cls.methods:
                    # Check if __init__ already exists
                    has_init = any(m[0] == "__init__" for m in existing.methods)
                    for m in cls.methods:
                        if m[0] == "__init__" and has_init:
                            continue
                        if not any(em[0] == m[0] for em in existing.methods):
                            existing.methods.append(m)
                if cls.getters:
                    for g in cls.getters:
                        if not any(eg[0] == g[0] for eg in existing.getters):
                            existing.getters.append(g)
                if cls.setters:
                    for s in cls.setters:
                        if not any(es[0] == s[0] for es in existing.setters):
                            existing.setters.append(s)
                if cls.static_methods:
                    for sm in cls.static_methods:
                        if not any(esm[0] == sm[0] for esm in existing.static_methods):
                            existing.static_methods.append(sm)
            else:
                all_classes[class_name] = cls

    # Sort by name
    return sorted(all_classes.values(), key=lambda c: c.name)


# ══════════════════════════════════════════════════════════════════
# Stub Generator
# ══════════════════════════════════════════════════════════════════

def generate_class_stub(cls: RustClass) -> list[str]:
    """Generate .pyi stub lines for a single class."""
    lines: list[str] = []

    if cls.doc:
        lines.append(f'"""{cls.doc}"""')

    if cls.is_enum:
        lines.append(f"class {cls.name}:")
        for variant in cls.enum_variants:
            lines.append(f"{INDENT}{variant}: ClassVar[{cls.name}]")
        return lines

    lines.append(f"class {cls.name}:")

    has_content = False

    # Fields (from #[pyo3(get)] / #[pyo3(get, set)])
    for field_name, field_type, has_set in cls.fields:
        lines.append(f"{INDENT}{field_name}: {field_type}")
        has_content = True

    # Getters (from #[getter] methods)
    for getter_name, ret_type in cls.getters:
        lines.append(f"{INDENT}@property")
        lines.append(f"{INDENT}def {getter_name}(self) -> {ret_type}: ...")
        has_content = True

    # Setters (from #[setter] methods)
    for setter_name, param_type in cls.setters:
        lines.append(f"{INDENT}@{setter_name}.setter")
        lines.append(f"{INDENT}def {setter_name}(self, value: {param_type}) -> None: ...")
        has_content = True

    # __init__ (from #[new])
    init_methods = [m for m in cls.methods if m[0] == "__init__"]
    if init_methods:
        _, ret, params = init_methods[0]
        param_parts = []
        for pname, ptype, pdefault in params:
            if pdefault is not None:
                # Convert Rust defaults to Python
                py_default = _convert_default(pdefault, ptype)
                param_parts.append(f"{pname}: {ptype} = {py_default}")
            else:
                param_parts.append(f"{pname}: {ptype}")
        lines.append(f"{INDENT}def __init__(self, {', '.join(param_parts)}) -> None: ...")
        has_content = True
    else:
        # No __init__ found — PyO3 classes without #[new] can't be instantiated
        # but we still show the class
        pass

    # Static methods
    for sm_name, sm_ret, sm_params in cls.static_methods:
        param_parts = []
        for pname, ptype, pdefault in sm_params:
            if pdefault is not None:
                py_default = _convert_default(pdefault, ptype)
                param_parts.append(f"{pname}: {ptype} = {py_default}")
            else:
                param_parts.append(f"{pname}: {ptype}")
        lines.append(f"{INDENT}@staticmethod")
        lines.append(f"{INDENT}def {sm_name}({', '.join(param_parts)}) -> {sm_ret}: ...")
        has_content = True

    # Regular methods (excluding __init__)
    for m_name, m_ret, m_params in cls.methods:
        if m_name == "__init__":
            continue
        param_parts = []
        for pname, ptype, pdefault in m_params:
            if pdefault is not None:
                py_default = _convert_default(pdefault, ptype)
                param_parts.append(f"{pname}: {ptype} = {py_default}")
            else:
                param_parts.append(f"{pname}: {ptype}")
        lines.append(f"{INDENT}def {m_name}(self, {', '.join(param_parts)}) -> {m_ret}: ...")
        has_content = True

    if not has_content:
        lines.append(f"{INDENT}pass")

    return lines


def _convert_default(rust_default: str, py_type: str) -> str:
    """Convert a Rust default value to its Python equivalent."""
    d = rust_default.strip()
    if d == "None":
        return "None"
    if d == "true":
        return "True"
    if d == "false":
        return "False"
    if d.startswith('"') and d.endswith('"'):
        return d
    if d.startswith("'") and d.endswith("'"):
        return d
    if d.isdigit() or (d.startswith("-") and d[1:].isdigit()):
        return d
    if d == "...":
        return "..."
    # For complex defaults, use ...
    return "..."


def generate_module_stub(module_name: str, classes: list[RustClass]) -> list[str]:
    """Generate .pyi stub lines for a module."""
    lines: list[str] = [f"# ── _tryx.{module_name} ──", ""]

    for cls in classes:
        lines.extend(generate_class_stub(cls))
        lines.append("")

    return lines


# ══════════════════════════════════════════════════════════════════
# Main
# ══════════════════════════════════════════════════════════════════

def main() -> int:
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

    for submod_name in sorted(MODULE_SOURCES.keys()):
        print(f"✓  Parsing _tryx.{submod_name}")
        classes = parse_rust_sources(submod_name)
        if classes:
            parts.extend(generate_module_stub(submod_name, classes))
            parts.append("")

    OUTPUT.write_text("\n".join(parts))
    size_kb = OUTPUT.stat().st_size / 1024
    n_classes = sum(1 for l in parts if l.strip().startswith("class "))
    n_methods = sum(1 for l in parts if "def " in l and "(self" in l)
    print(f"\n✅ {OUTPUT.name}: {size_kb:.1f} KB, {n_classes} classes, {n_methods} methods")
    return 0


if __name__ == "__main__":
    sys.exit(main())
