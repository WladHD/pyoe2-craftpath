#!/usr/bin/env bash
# Seed the offline bench cache with the CraftOfExile + PoE Ninja JSON the CLI
# normally fetches itself. Prefers copying the (gitignored) example cache at
# backend/python_examples/cache/ and only goes online when that is absent.
#
# Env knobs:
#   CACHE_DIR     where to put the files     (default: <repo>/.bench/cache)
#   POE2_LEAGUE   league for PoE Ninja URLs  (default: Standard)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CACHE_DIR="${CACHE_DIR:-$REPO_ROOT/.bench/cache}"
POE2_LEAGUE="${POE2_LEAGUE:-Standard}"
SEED_DIR="$REPO_ROOT/backend/python_examples/cache"

FILES=(coe2.json pn_abyss.json pn_currency.json pn_essences.json pn_ritual.json)

die() { echo "warm_cache: $*" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || die "jq is required"

have_all_seeds() {
    local f
    for f in "${FILES[@]}"; do [ -f "$SEED_DIR/$f" ] || return 1; done
}

url_for() {
    case "$1" in
        coe2.json) echo "https://www.craftofexile.com/json/poe2/main/poec_data.json" ;;
        pn_abyss.json) echo "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=${POE2_LEAGUE}&type=Abyss" ;;
        pn_currency.json) echo "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=${POE2_LEAGUE}&type=Currency" ;;
        pn_essences.json) echo "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=${POE2_LEAGUE}&type=Essences" ;;
        pn_ritual.json) echo "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=${POE2_LEAGUE}&type=Ritual" ;;
        *) die "unknown cache file '$1'" ;;
    esac
}

mkdir -p "$CACHE_DIR"

if have_all_seeds; then
    echo "warm_cache: seeding from $SEED_DIR"
    for f in "${FILES[@]}"; do cp "$SEED_DIR/$f" "$CACHE_DIR/$f"; done
else
    echo "warm_cache: example cache absent, downloading (league: $POE2_LEAGUE)"
    for f in "${FILES[@]}"; do
        curl -fsSL "$(url_for "$f")" -o "$CACHE_DIR/$f" || die "download of $f failed"
    done
fi

# coe2.json ships as `poecd={...}` (the Rust parser strips that prefix).
sed 's/^poecd=//' "$CACHE_DIR/coe2.json" | jq -e . >/dev/null \
    || die "$CACHE_DIR/coe2.json is not valid JSON (after stripping 'poecd=')"
for f in "${FILES[@]:1}"; do
    jq -e . "$CACHE_DIR/$f" >/dev/null || die "$CACHE_DIR/$f is not valid JSON"
done

echo "warm_cache: cache ready at $CACHE_DIR"
