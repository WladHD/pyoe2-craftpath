#!/usr/bin/env bash
# Run the python gate: build the extension into the dev venv (via
# setup_python_env.sh), run the python test suite, then clean and re-execute
# the example notebooks in place (refreshing their outputs). Any notebook
# cell error fails the script. Called by the husky pre-push hook; also
# runnable standalone.
#
# Env knobs:
#   SKIP_NOTEBOOKS=1   run pytest only, leave the notebooks untouched
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CRATE_DIR="$REPO_ROOT/backend/crates/pyoe2-craftpath"
EXAMPLES_DIR="$REPO_ROOT/backend/python_examples"
VENV="$CRATE_DIR/.venv"

"$REPO_ROOT/backend/scripts/setup_python_env.sh"
export VIRTUAL_ENV="$VENV"
export PATH="$VENV/bin:$PATH"

cd "$CRATE_DIR"
python -m pytest python/tests -q

if [ -z "${SKIP_NOTEBOOKS:-}" ]; then
    # clean + rerun every notebook in place; --allow-errors lets the run
    # finish so every traceback is captured in the notebook outputs, then
    # any cell error fails the script (and aborts the push)
    for nb in "$EXAMPLES_DIR"/*.ipynb; do
        echo "Cleaning + executing $(basename "$nb")"
        python -m nbconvert --to notebook --clear-output --inplace "$nb"
        python -m nbconvert --to notebook --execute --allow-errors --inplace "$nb"
    done

    failing=$(grep -l '"output_type": "error"' "$EXAMPLES_DIR"/*.ipynb || true)
    if [ -n "$failing" ]; then
        echo "ERROR: notebooks reran with cell errors (tracebacks kept in their outputs):"
        echo "$failing"
        exit 1
    fi
fi
