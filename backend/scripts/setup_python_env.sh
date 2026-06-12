#!/usr/bin/env bash
# Create or refresh the python dev venv at backend/crates/pyoe2-craftpath/.venv:
# builds the extension into it via maturin and installs the pinned dev
# requirements. Idempotent - rerun any time. Source of truth for dependencies
# is backend/requirements-dev.txt.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CRATE_DIR="$REPO_ROOT/backend/crates/pyoe2-craftpath"
VENV="$CRATE_DIR/.venv"

cd "$CRATE_DIR"

[ -d "$VENV" ] || uv venv "$VENV"
export VIRTUAL_ENV="$VENV"
export PATH="$VENV/bin:$PATH"

# maturin develop also installs the package's runtime deps (e.g. polars)
uvx maturin develop --uv
uv pip install --quiet -r "$REPO_ROOT/backend/requirements-dev.txt"

echo "python dev venv ready at $VENV"
