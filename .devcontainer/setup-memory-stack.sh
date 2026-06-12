#!/usr/bin/env bash
# Idempotent bootstrap for the 4-layer memory/knowledge stack.
# Re-runs cleanly on every container creation (postCreateCommand).
#
#   L1 - auto-memory at /workspaces/racoon/.claude-memory (symlinked into $CLAUDE_CONFIG_DIR)
#   L2 - LLM Wiki plugin (praneybehl/llm-wiki-plugin); wiki lives at /workspaces/racoon/wiki (git-tracked; scaffold with /wiki:init)
#   L3 - mempalace (palace data at /workspaces/racoon/.mempalace/palace, workspace-survivable)
#   L4 - memsearch plugin (zilliztech/memsearch)
#
# See: /workspaces/racoon/CLAUDE.md → "Memory layout" section.

set -euo pipefail

export PATH="/root/.local/bin:${PATH}"

# Claude Code's config dir, relocated into the bind-mounted workspace by the
# devcontainer's containerEnv (CLAUDE_CONFIG_DIR) so auth / plugins / settings /
# transcripts persist across rebuilds. Falls back to the default if unset.
export CLAUDE_CONFIG_DIR="${CLAUDE_CONFIG_DIR:-/root/.claude}"
mkdir -p "$CLAUDE_CONFIG_DIR"

log() { printf '  %s\n' "$*"; }
section() { printf '\n→ %s\n' "$*"; }

# ---------------------------------------------------------------------------
# L3 - mempalace
# ---------------------------------------------------------------------------
section "L3 mempalace"

# Ensure the mempalace CLI exists. It's baked into the image (Dockerfile:
# `uv tool install mempalace`), but self-heal here so L3 still works if this
# script runs against an older image. `command -v` in an `if` is safe under
# `set -e`; the `|| log` keeps a transient install failure non-fatal.
if ! command -v mempalace >/dev/null 2>&1; then
    log "installing mempalace CLI (uv tool install)"
    uv tool install mempalace >/dev/null 2>&1 || log "warning: mempalace install failed - L3 will no-op until installed"
fi

# Ensure global config points the palace at /workspaces/racoon/ so data survives rebuilds.
mkdir -p /root/.mempalace
if [ -f /root/.mempalace/config.json ]; then
    python3 - <<'PY'
import json, pathlib
p = pathlib.Path("/root/.mempalace/config.json")
c = json.loads(p.read_text())
c["palace_path"] = "/workspaces/racoon/.mempalace/palace"
p.write_text(json.dumps(c, indent=2))
PY
else
    cat > /root/.mempalace/config.json <<'JSON'
{
  "palace_path": "/workspaces/racoon/.mempalace/palace",
  "collection_name": "mempalace_drawers"
}
JSON
fi
log "palace_path → /workspaces/racoon/.mempalace/palace"

mkdir -p /workspaces/racoon/.mempalace/palace
# /workspaces/racoon/mempalace.yaml is the wing+rooms config - git-tracked, owned by user.
# mempalace looks for it at the mine-dir root, not inside .mempalace/.
if [ ! -f /workspaces/racoon/mempalace.yaml ]; then
    log "warning: /workspaces/racoon/mempalace.yaml missing - mine will use defaults (wing='workspace', room='general')"
fi

# ---------------------------------------------------------------------------
# L1 - Auto-memory symlinked into $CLAUDE_CONFIG_DIR so files live in /workspaces/racoon/
# ---------------------------------------------------------------------------
section "L1 auto-memory"

mkdir -p /workspaces/racoon/.claude-memory
mkdir -p "$CLAUDE_CONFIG_DIR/projects/-workspaces-racoon"
link_target="/workspaces/racoon/.claude-memory"
link_path="$CLAUDE_CONFIG_DIR/projects/-workspaces-racoon/memory"
if [ -L "$link_path" ] && [ "$(readlink "$link_path")" = "$link_target" ]; then
    log "symlink already correct"
else
    rm -rf "$link_path"
    ln -s "$link_target" "$link_path"
    log "$link_path → $link_target"
fi

# ---------------------------------------------------------------------------
# L2 + L4 - Plugin installs (via the `claude plugin` CLI)
# ---------------------------------------------------------------------------
# Plugins live under $CLAUDE_CONFIG_DIR/plugins - now in the bind-mounted workspace
# (CLAUDE_CONFIG_DIR set by the devcontainer) - so this is a one-time install and a
# cheap no-op on every rebuild thereafter. The CLI reads each repo's marketplace.json
# (which declares the plugin's source subpath) and writes installed_plugins.json /
# known_marketplaces.json / enabledPlugins itself, so no hand-cloning or JSON
# patching is needed.
section "L2/L4 plugins"

# Idempotent: skip if already installed (persisted in the workspace), else add the
# marketplace, install, and enable. Non-fatal on failure (offline / transient);
# falls back to a bare `install <plugin>` if the plugin@marketplace name doesn't
# resolve. Retries on the next rebuild.
ensure_plugin() {  # plugin_name  marketplace_repo  marketplace_name
    local plugin="$1" repo="$2" mkt="$3"
    if claude plugin list 2>/dev/null | grep -q "${plugin}"; then
        log "${plugin} already installed"
    else
        log "installing ${plugin} (marketplace ${repo})"
        claude plugin marketplace add "${repo}" >/dev/null 2>&1 || true
        claude plugin install "${plugin}@${mkt}" --scope user >/dev/null 2>&1 \
            || claude plugin install "${plugin}" --scope user >/dev/null 2>&1 \
            || log "warning: ${plugin} install failed - will retry next rebuild"
    fi
    # Always (re-)enable - idempotent, and guarantees the plugin loads even when a
    # persisted/pre-seeded install left it present-but-disabled in settings.json.
    claude plugin enable "${plugin}" >/dev/null 2>&1 || true
}

# L4 - memsearch (transcript-recall backstop)
ensure_plugin memsearch zilliztech/memsearch     memsearch-plugins

# L2 - LLM Wiki (curated knowledge base)
ensure_plugin llm-wiki  praneybehl/llm-wiki-plugin llm-wiki

# ---------------------------------------------------------------------------
# Patch $CLAUDE_CONFIG_DIR/settings.json - register the two custom mempalace hooks.
# Plugin enable + marketplace registration is handled by `claude plugin` above;
# the mempalace hooks are NOT plugin-supplied, so we add them here. Preserves
# permissions and any other existing keys. Persisted in the workspace via CLAUDE_CONFIG_DIR.
# ---------------------------------------------------------------------------
section "settings.json (mempalace hooks)"

mkdir -p "$CLAUDE_CONFIG_DIR"
python3 - <<'PY'
import json, os, pathlib
p = pathlib.Path(os.environ.get("CLAUDE_CONFIG_DIR", "/root/.claude")) / "settings.json"
s = json.loads(p.read_text()) if p.exists() else {}

# Hooks - mempalace L3 auto-sync.
# Plugin-supplied hooks (memsearch, llm-wiki) are discovered automatically via
# their own hooks.json files using ${CLAUDE_PLUGIN_ROOT}; we don't duplicate.
def upsert_hook(event, marker, entry):
    arr = s.setdefault("hooks", {}).setdefault(event, [])
    # Replace any existing entry whose command contains `marker` (so script
    # updates to the command propagate on rebuild). Otherwise append.
    for i, existing in enumerate(arr):
        for h in existing.get("hooks", []):
            if marker in h.get("command", ""):
                arr[i] = entry
                return
    arr.append(entry)

# SessionStart: emit mempalace wake-up context (~30-900 tokens).
# No `matcher` field - matches memsearch's working pattern. With matcher=""
# the hook silently never fires on real session-start events. sed strips the
# noisy "Wake-up text (~N tokens):" header + the ====== separator so only
# the actual L0/L1 content flows into the context.
upsert_hook("SessionStart", "mempalace wake-up", {
    "hooks": [{
        "type": "command",
        "command": "mempalace wake-up 2>/dev/null | sed -e '1,2d' -e '/^=\\+$/d' || true",
        "timeout": 15,
    }],
})

# PostToolUse: incremental re-mine in background after edits.
# Idle re-mine ≈ 1.2s; first-time mining ≈ 3s/file. flock prevents concurrent
# mines from racing on the chroma sqlite DB when edits arrive rapidly.
# -n: non-blocking - if a mine is already running, skip this trigger silently.
upsert_hook("PostToolUse", "mempalace mine /workspaces/racoon", {
    "matcher": "Edit|Write|MultiEdit",
    "hooks": [{
        "type": "command",
        "command": "( nohup flock -n /tmp/mempalace-mine.lock mempalace mine /workspaces/racoon </dev/null >/dev/null 2>&1 & disown ) 2>/dev/null || true",
    }],
})

p.write_text(json.dumps(s, indent=2))
PY
log "mempalace hooks patched in (plugins enabled via the claude plugin CLI)"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "✓ Memory stack ready."
echo "  Config dir:     $CLAUDE_CONFIG_DIR (workspace-persistent across rebuilds)"
echo "  L1 auto-memory: /workspaces/racoon/.claude-memory"
echo "  L2 LLM Wiki:    /workspaces/racoon/wiki/ (git-tracked; /wiki:init is idempotent)"
echo "  L3 mempalace:   /workspaces/racoon/.mempalace/  (palace data at .mempalace/palace)"
echo "  L4 memsearch:   installed via 'claude plugin' (see: claude plugin list)"
echo ""
echo "  First-time mine (background): mempalace mine /workspaces/racoon"
