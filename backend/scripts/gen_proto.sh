#!/usr/bin/env bash
# Regenerate the committed Python protobuf code in
# backend/crates/pyoe2-craftpath/python/pyoe2_craftpath/_proto/
# from the shared schemas in /proto. Requires protoc >= 25.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PROTO_DIR="$REPO_ROOT/proto"
OUT_DIR="$REPO_ROOT/backend/crates/pyoe2-craftpath/python/pyoe2_craftpath/_proto"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

protoc -I "$PROTO_DIR" \
    --python_out="$TMP_DIR" \
    --pyi_out="$TMP_DIR" \
    "$PROTO_DIR"/craftpath/v1/*.proto

mkdir -p "$OUT_DIR"
rm -f "$OUT_DIR"/*_pb2.py "$OUT_DIR"/*_pb2.pyi

# Flatten craftpath/v1/*_pb2.py into the package and rewrite the absolute
# imports (`from craftpath.v1 import x_pb2 as ...`) to relative ones.
for f in "$TMP_DIR"/craftpath/v1/*_pb2.py "$TMP_DIR"/craftpath/v1/*_pb2.pyi; do
    base="$(basename "$f")"
    sed -E 's/^from craftpath\.v1 import /from . import /' "$f" > "$OUT_DIR/$base"
done

cat > "$OUT_DIR/__init__.py" <<'EOF'
"""Generated protobuf code for craftpath.v1 (see /proto). Regenerate with
backend/scripts/gen_proto.sh — do not edit by hand."""

from . import common_pb2, currency_pb2, item_pb2, job_pb2, presets_pb2  # noqa: F401

# convenience re-export: everything clients typically need lives in job_pb2
from .job_pb2 import *  # noqa: F401,F403
EOF

echo "generated into $OUT_DIR"
ls "$OUT_DIR"
