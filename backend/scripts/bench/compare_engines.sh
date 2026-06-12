#!/usr/bin/env bash
# Benchmark the pinned baseline engine (default: tag 0.5.1, the last release
# before the dev-rework) against the current working tree. Builds both CLIs,
# runs every case from backend/benches/cases.json offline against a pre-warmed
# API cache, and writes a timestamped result JSON plus a regenerated latest.md
# into backend/benches/results/.
#
# The crate moved from the repo root into a backend/ cargo workspace during
# the rework, so manifest location, binary name and invocation differ per
# revision - build_cli() resolves all three.
#
# Env knobs:
#   BASELINE_REF  git rev of the old engine          (default: 0.5.1)
#   HEAD_REF      git rev to measure as "head"       (default: the working tree)
#   RUNS          timed runs per binary per case     (default: 5)
#   BENCH_DIR     scratch: baseline worktree + cache (default: <repo>/.bench)
#   RESULTS_DIR   committed results dir              (default: backend/benches/results)
#   CASES_FILE    bench case manifest                (default: backend/benches/cases.json)
#   POE2_LEAGUE   league used when warming the cache (default: Standard)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BASELINE_REF="${BASELINE_REF:-0.5.1}"
RUNS="${RUNS:-5}"
BENCH_DIR="${BENCH_DIR:-$REPO_ROOT/.bench}"
RESULTS_DIR="${RESULTS_DIR:-$REPO_ROOT/backend/benches/results}"
CASES_FILE="${CASES_FILE:-$REPO_ROOT/backend/benches/cases.json}"
export POE2_LEAGUE="${POE2_LEAGUE:-Standard}"

die() { echo "compare_engines: $*" >&2; exit 1; }

command -v jq >/dev/null 2>&1 || die "jq is required"
command -v cargo >/dev/null 2>&1 || die "cargo is required"
[ -f "$CASES_FILE" ] || die "cases file not found: $CASES_FILE"

BASELINE_SHA="$(git -C "$REPO_ROOT" rev-parse --verify "${BASELINE_REF}^{commit}")" \
    || die "cannot resolve BASELINE_REF '$BASELINE_REF'"

# ---------------------------------------------------------------------------
# Builds the CLI of a checkout and prints the command prefix to invoke it.
#   workspace layout (backend/crates): `pyoe2-backend cli` subcommand
#   legacy single-crate layout (root): `pyoe2_craftpath_cli` binary
build_cli() { # $1 = checkout dir, rest = extra cargo flags
    local root="$1" manifest dir
    shift
    if [ -f "$root/backend/Cargo.toml" ]; then manifest="$root/backend/Cargo.toml"
    elif [ -f "$root/Cargo.toml" ]; then manifest="$root/Cargo.toml"
    else die "no Cargo.toml found under $root"; fi
    dir="$(dirname "$manifest")"
    if [ -d "$dir/crates/craftpath-server" ]; then
        cargo build --release --manifest-path "$manifest" -p craftpath-server \
            --no-default-features "$@" >&2 \
            || die "cargo build failed in $dir"
        echo "$dir/target/release/pyoe2-backend cli"
    else
        cargo build --release --manifest-path "$manifest" \
            --no-default-features "$@" >&2 \
            || die "cargo build failed in $dir"
        echo "$dir/target/release/pyoe2_craftpath_cli"
    fi
}

# --- builds -----------------------------------------------------------------
# Pinned revisions get a sha-keyed cached worktree under BENCH_DIR, so
# baseline and HEAD_REF share one checkout + build when they point at the
# same commit (e.g. a baseline self-run with HEAD_REF=<baseline sha>).
rev_worktree() { # $1 = commit sha; prints the worktree dir
    local dir
    dir="$BENCH_DIR/rev-$(git -C "$REPO_ROOT" rev-parse --short "$1")"
    if [ ! -d "$dir" ]; then
        mkdir -p "$BENCH_DIR"
        git -C "$REPO_ROOT" worktree add --detach "$dir" "$1" >&2
    fi
    echo "$dir"
}

echo "==> building baseline CLI ($BASELINE_REF @ ${BASELINE_SHA:0:7}) ..." >&2
# --locked: pinned revisions must build with their own committed Cargo.lock
BASELINE_CMD="$(build_cli "$(rev_worktree "$BASELINE_SHA")" --locked)"

if [ -n "${HEAD_REF:-}" ]; then
    HEAD_SHA="$(git -C "$REPO_ROOT" rev-parse --verify "${HEAD_REF}^{commit}")" \
        || die "cannot resolve HEAD_REF '$HEAD_REF'"
    echo "==> building head CLI ($HEAD_REF @ ${HEAD_SHA:0:7}) ..." >&2
    HEAD_CMD="$(build_cli "$(rev_worktree "$HEAD_SHA")" --locked)"
    HEAD_DIRTY="false"
else
    HEAD_SHA="$(git -C "$REPO_ROOT" rev-parse HEAD)"
    echo "==> building head CLI (working tree) ..." >&2
    HEAD_CMD="$(build_cli "$REPO_ROOT")"
    HEAD_DIRTY="false"
    if [ -n "$(git -C "$REPO_ROOT" status --porcelain --untracked-files=no)" ]; then
        HEAD_DIRTY="true"
    fi
fi

# --- cache ------------------------------------------------------------------
CACHE="$BENCH_DIR/cache"
if [ ! -f "$CACHE/coe2.json" ]; then
    CACHE_DIR="$CACHE" "$REPO_ROOT/backend/scripts/bench/warm_cache.sh" >&2
fi
# Refresh mtimes so the CLIs' cache TTL (1 h for pn_*.json) can never trigger
# a network download mid-bench.
touch "$CACHE"/*.json

# --- helpers ----------------------------------------------------------------
cli_args() { # $1 = start item, $2 = target item
    echo "--start-item-path $1 --target-item-path $2 --cache-path $CACHE \
--amount-routes 5 --poe2-league $POE2_LEAGUE --no-updates --no-groups --max-ram 1G"
}

# One untimed run per binary x case: warms the binary and checks correctness.
# The CLI always exits 0 (it logs errors and waits for Enter), so the output
# has to be grepped for tracing ERROR lines.
sanity_run() { # $1 = cmd prefix, $2 = start, $3 = target, $4 = label
    local out
    out="$(echo | $1 $(cli_args "$2" "$3") 2>&1)" || die "$4 crashed on $2"
    if printf '%s' "$out" | grep -q "ERROR"; then
        printf '%s\n' "$out" | tail -n 20 >&2
        die "$4 reported a calculation ERROR for $2 -> $3"
    fi
}

# Times RUNS executions; prints "mean min max" in seconds.
time_runs() { # $1 = cmd prefix, $2 = start, $3 = target
    if command -v hyperfine >/dev/null 2>&1; then
        local tmp
        tmp="$(mktemp)"
        hyperfine --runs "$RUNS" --style none --export-json "$tmp" \
            "echo | $1 $(cli_args "$2" "$3") >/dev/null 2>&1" >&2
        jq -r '.results[0] | "\(.mean) \(.min) \(.max)"' "$tmp"
        rm -f "$tmp"
    else
        local samples=() t0 t1 i
        for i in $(seq 1 "$RUNS"); do
            t0=$(date +%s%N)
            echo | $1 $(cli_args "$2" "$3") >/dev/null 2>&1
            t1=$(date +%s%N)
            samples+=($((t1 - t0)))
        done
        printf '%s\n' "${samples[@]}" | awk '
            { total += $1; if (min == "" || $1 < min) min = $1; if ($1 > max) max = $1 }
            END { printf "%.3f %.3f %.3f\n", total / NR / 1e9, min / 1e9, max / 1e9 }'
    fi
}

# --- run all cases ----------------------------------------------------------
command -v hyperfine >/dev/null 2>&1 && TIMER="hyperfine" || TIMER="bash"
TMP_CASES="$(mktemp)"
trap 'rm -f "$TMP_CASES"' EXIT

while IFS=$'\t' read -r name start target; do
    start="$REPO_ROOT/backend/$start"
    target="$REPO_ROOT/backend/$target"
    [ -f "$start" ] || die "start item missing: $start"
    [ -f "$target" ] || die "target item missing: $target"

    echo "==> case '$name': sanity runs ..." >&2
    sanity_run "$BASELINE_CMD" "$start" "$target" "baseline"
    sanity_run "$HEAD_CMD" "$start" "$target" "head"

    echo "==> case '$name': timing baseline ($RUNS runs) ..." >&2
    read -r b_mean b_min b_max <<<"$(time_runs "$BASELINE_CMD" "$start" "$target")"
    echo "==> case '$name': timing head ($RUNS runs) ..." >&2
    read -r h_mean h_min h_max <<<"$(time_runs "$HEAD_CMD" "$start" "$target")"

    jq -n --arg name "$name" \
        --argjson bm "$b_mean" --argjson bmin "$b_min" --argjson bmax "$b_max" \
        --argjson hm "$h_mean" --argjson hmin "$h_min" --argjson hmax "$h_max" \
        '{name: $name,
          baseline_s: {mean: $bm, min: $bmin, max: $bmax},
          head_s: {mean: $hm, min: $hmin, max: $hmax},
          speedup: (if $hm > 0 then ($bm / $hm) else null end)}' >>"$TMP_CASES"
done < <(jq -r '.[] | [.name, .start, .target] | @tsv' "$CASES_FILE")

# --- result files -----------------------------------------------------------
# HEAD_SHA / HEAD_DIRTY were resolved in the builds section (pinned HEAD_REF
# or the working tree).
mkdir -p "$RESULTS_DIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
CPU="$(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2- | xargs || uname -m)"
RESULT_FILE="$RESULTS_DIR/${STAMP}_${HEAD_SHA:0:7}.json"

jq -s --arg timestamp "$STAMP" \
    --arg baseline_ref "$BASELINE_REF" --arg baseline_sha "$BASELINE_SHA" \
    --arg head_ref "${HEAD_REF:-worktree}" \
    --arg head_sha "$HEAD_SHA" --argjson head_dirty "$HEAD_DIRTY" \
    --argjson runs "$RUNS" --arg timer "$TIMER" \
    --arg host "$(uname -n)" --arg cpu "$CPU" --arg league "$POE2_LEAGUE" \
    '{meta: {timestamp: $timestamp, baseline_ref: $baseline_ref,
             baseline_sha: $baseline_sha, head_ref: $head_ref,
             head_sha: $head_sha, head_dirty: $head_dirty, runs: $runs,
             timer: $timer, host: $host, cpu: $cpu, league: $league},
      cases: (map({(.name): del(.name)}) | add)}' "$TMP_CASES" >"$RESULT_FILE"

{
    echo "# Engine benchmark - latest run"
    echo
    echo "Generated by \`backend/scripts/bench/compare_engines.sh\` from" \
         "\`$(basename "$RESULT_FILE")\` - do not edit by hand."
    echo
    jq -r '.meta |
        "- baseline: `\(.baseline_ref)` (`\(.baseline_sha[0:7])`)\n" +
        "- head: `\(.head_sha[0:7])`" + (if .head_dirty then " (dirty working tree)" else "" end) + "\n" +
        "- \(.runs) runs each, timer: \(.timer), host: \(.host), cpu: \(.cpu)\n" +
        "- UTC: \(.timestamp), league: \(.league)"' "$RESULT_FILE"
    echo
    echo "| case | baseline mean (s) | head mean (s) | speedup |"
    echo "| --- | ---: | ---: | ---: |"
    jq -r '.cases | to_entries[] |
        "| \(.key) | \(.value.baseline_s.mean | . * 1000 | round / 1000) | \(.value.head_s.mean | . * 1000 | round / 1000) | \(.value.speedup | . * 100 | round / 100)x |"' \
        "$RESULT_FILE"
} >"$RESULTS_DIR/latest.md"

echo
echo "compare_engines: wrote $RESULT_FILE"
echo
cat "$RESULTS_DIR/latest.md"
