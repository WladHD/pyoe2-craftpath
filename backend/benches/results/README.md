# Benchmark results

Output of `backend/scripts/bench/compare_engines.sh`, which times full CLI
runs of the pinned baseline engine (tag `0.5.1`, the last release before the
dev-rework) against the current working tree over the item pairs defined in
`backend/benches/cases.json`.

Conventions:

- One immutable `YYYYMMDDTHHMMSSZ_<shortsha>.json` per harness run; never
  edit or rewrite old files.
- `latest.md` is regenerated on every run. On a merge conflict, rerun the
  harness instead of hand-merging.
- Numbers are machine-relative - the `meta` block records host/CPU, so only
  compare runs from the same machine.
- The criterion benches (`cargo bench -p craftpath-core`) keep their raw
  data untracked in `backend/target/criterion/`.

The pre-push hook (`.githooks/pre-push`, install via `.githooks/install.sh`)
runs the harness automatically when pushed commits touch `backend/**` and
commits new result files here as their own commit.
