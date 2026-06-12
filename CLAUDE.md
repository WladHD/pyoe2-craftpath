# CLAUDE.md

## Memory layout

Four layers, one tool each. Each layer answers a distinct question - pick the right one before writing. Details: [wiki/concepts/memory-stack.md](wiki/concepts/memory-stack.md).

| Layer              | Question it answers                                                                 | Where to write                                                                                                                                                                      |
| ------------------ | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **L1 Auto-memory** | "What does Claude need to remember about _me_ and _this project's standing rules_?" | `$CLAUDE_CONFIG_DIR/projects/-workspace/memory/`. One file per fact, typed `user`/`feedback`/`project`/`reference`. Index in `MEMORY.md`.                                            |
| **L2 LLM Wiki**    | "What did we decide and why? What broke and how did we fix it?"                     | `wiki/` - `sources/`, `entities/`, `concepts/`, `synthesis/`. Hand-curated. Use `/wiki:ingest`, `/wiki:query`, `/wiki:lint`. Raw source material into `raw/`.                        |
| **L3 mempalace**   | "What does the _current_ codebase look like? Give me a wake-up summary."            | `.mempalace/` - auto-mined. `mempalace search`, `mempalace wake-up`. **Not yet configured for this repo** - see the memory-stack wiki page before relying on it.                     |
| **L4 memsearch**   | "What did we discuss in past sessions about X?"                                     | Auto-indexed daily from transcripts. `/memsearch:memory-recall` skill. Zero curation.                                                                                               |

## Repo layout

The Rust + PyO3 core (CLI, Python package `pyoe2-craftpath`) lives under `backend/` - build and test from there (`cargo test`, `maturin develop` / `uv`). `backend/scripts/setup_python_env.sh` creates the python dev venv (deps pinned in `backend/requirements-dev.txt`). `frontend/` is an empty placeholder for the upcoming rework.

Benchmarks: criterion benches live in `backend/crates/craftpath-core/benches/`, the baseline-vs-head CLI harness in `backend/scripts/bench/`, committed results in `backend/benches/results/`. Run `npm install` once per clone to activate the husky pre-push hook (`.husky/pre-push`) that regenerates the python stubs, runs the rust and python tests, reruns the example notebooks and benches `backend/**` changes automatically.

## Commit messages

Format: `<type>(<scope>): <:gitmoji:> <subject>` - scope optional; omit for repo-wide changes (`<type>: <:gitmoji:> <subject>`). Version bumps use the exact form `chore: :bookmark: bump version to <X.Y.Z>` and change only the version files.

See [wiki/concepts/commit-conventions.md](wiki/concepts/commit-conventions.md) for the requirements - the type/scope/gitmoji/subject rules, the version-bump form, and the full gitmoji shortcode table.
