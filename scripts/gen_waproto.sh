#!/usr/bin/env bash
#
# Regenerate the Python protobuf bindings for waproto from the upstream
# whatsapp-rust proto definitions.
#
#   source : libs/whatsapp-rust/waproto/src/whatsapp.proto   (git submodule)
#   outputs: python/tryx/waproto/whatsapp_pb2.py             (protoc --python_out)
#            python/tryx/waproto/whatsapp_pb2.pyi            (mypy-protobuf --mypy_out)
#
# The generated descriptor/source name is kept as "waproto/whatsapp.proto"
# (matching the previously checked-in gencode, so imports and the embedded
# descriptor filename stay stable) by staging the proto under that relative
# path before invoking protoc.
#
# Requirements:
#   * protoc            — the Protocol Buffers compiler (system package)
#   * protoc-gen-mypy   — provided by the `mypy-protobuf` dev dependency;
#                         activate the project venv, or `pip install mypy-protobuf`
#
# NOTE: protoc emits a gencode runtime-version guard. Keep the installed
# `protobuf` runtime >= the protoc series (e.g. protoc 34.x -> protobuf 7.34.x),
# otherwise importing the generated module raises a VersionError.
#
# Usage: scripts/gen_waproto.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$REPO_ROOT/libs/whatsapp-rust/waproto/src/whatsapp.proto"
OUT_ROOT="$REPO_ROOT/python/tryx"   # canonical name prepends the waproto/ dir
CANONICAL="waproto/whatsapp.proto"

command -v protoc >/dev/null 2>&1 || {
  echo "error: 'protoc' not found in PATH" >&2
  exit 1
}
command -v protoc-gen-mypy >/dev/null 2>&1 || {
  echo "error: 'protoc-gen-mypy' not found — activate the venv or 'pip install mypy-protobuf'" >&2
  exit 1
}
[ -f "$SRC" ] || {
  echo "error: proto source not found: $SRC" >&2
  echo "hint: run 'git submodule update --init libs/whatsapp-rust'" >&2
  exit 1
}

# Stage the proto under waproto/ so protoc's canonical file name (and therefore
# the descriptor name and output path) become waproto/whatsapp.proto.
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/waproto"
cp "$SRC" "$STAGE/waproto/whatsapp.proto"

protoc \
  -I "$STAGE" \
  --python_out="$OUT_ROOT" \
  --mypy_out="$OUT_ROOT" \
  "$CANONICAL"

echo "OK — regenerated with $(protoc --version):" >&2
echo "  python/tryx/waproto/whatsapp_pb2.py" >&2
echo "  python/tryx/waproto/whatsapp_pb2.pyi" >&2
