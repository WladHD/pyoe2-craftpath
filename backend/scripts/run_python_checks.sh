#!/usr/bin/env bash
# Build the extension into a local venv, run the python test suite and
# re-execute the example notebooks in place (refreshing their outputs).
# Called by the husky pre-push hook; also runnable standalone.
#
# Env knobs:
#   SKIP_NOTEBOOKS=1   run pytest only, leave the notebooks untouched
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CRATE_DIR="$REPO_ROOT/backend/crates/pyoe2-craftpath"
EXAMPLES_DIR="$REPO_ROOT/backend/python_examples"
VENV="$CRATE_DIR/.venv"

cd "$CRATE_DIR"

[ -d "$VENV" ] || uv venv "$VENV"
export VIRTUAL_ENV="$VENV"
export PATH="$VENV/bin:$PATH"

# build + install the extension (maturin develop also installs the
# package's runtime deps), then the test/notebook tooling
uvx maturin develop --uv
uv pip install --quiet \
    pytest pytest-asyncio httpx websockets protobuf \
    pandas matplotlib nbconvert nbclient ipykernel

python -m pytest python/tests -q

if [ -z "${SKIP_NOTEBOOKS:-}" ]; then
    for nb in "$EXAMPLES_DIR"/*.ipynb; do
        echo "Executing $(basename "$nb")"
        python -m nbconvert --to notebook --execute --inplace "$nb"
    done
fi
