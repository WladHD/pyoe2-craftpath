#!/usr/bin/env bash
# Point git at the tracked hooks in .githooks/. Run once per clone.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
chmod +x .githooks/pre-push .githooks/install.sh backend/scripts/bench/*.sh
git config core.hooksPath .githooks
echo "core.hooksPath -> .githooks (pre-push bench hook active)"
