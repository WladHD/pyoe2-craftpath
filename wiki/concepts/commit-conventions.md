---
type: concept
title: "Commit conventions"
tags: [contribution]
sources: []
created: 2026-06-11
updated: 2026-06-11
---

# Commit conventions

Format: **conventional commits + gitmoji**, with a strong emphasis on
why-first bodies for substantive changes.

## Header

```
type(scope): :gitmoji: imperative subject
```

- **type**: `feat` | `fix` | `docs` | `test` | `chore` | `refactor` | `perf`
  | `style` | `ci`. Append `!` for breaking changes
  (`feat(api)!: ...`).
- **scope**: the feature/package touched - e.g. `api`, `calculator`,
  `example`, `proto`, `server`, `python`, `deploy`, `core`,
  `external_api`, `devcontainer`, `pages`. Combined scopes as
  `(api - cli)` when one change spans two. Optional for repo-wide changes.
- **gitmoji**: always the `:code:` text form, never the literal emoji
  (history mixes both; `:code:` is the convention going forward). Common
  pairs: `:sparkles:` feat, `:bug:` fix, `:memo:` docs,
  `:white_check_mark:` tests, `:bookmark:` version bump, `:zap:` perf,
  `:recycle:` refactor, `:building_construction:` structural moves,
  `:rocket:` deploy/CI, `:card_file_box:` data, `:lipstick:` UI,
  `:art:` formatting, `:pencil2:` typos, `:twisted_rightwards_arrows:`
  merges.
- **subject**: imperative mood, lower-case start, no trailing period.
  Quote domain terms in backticks (`` `Fracturing Orb` ``, `` `Standard` ``,
  `` `PropagationTarget` ``).

## Body (required for substantive commits)

- **Why first, then what**: open with the problem or motivation in plain
  prose ("A referenced questionnaire on a workflow step was a dead end -
  ..."), then describe the change. Use `- ` bullets when a commit has
  several distinct parts.
- Wrap at ~80 columns.
- State what was **verified** (tests run, suites green, measurements) -
  especially for perf or behavior-adjacent changes.
- **Transparency parentheticals**: if a hook/check was bypassed or could
  not run, say so and say what *was* checked, e.g.
  `(pre-commit bypassed: @vitest/browser cannot start headless; tsc +
  prettier clean.)`
- Trivial chores (version bumps, merges) need no body.

## Trailers

- AI-assisted commits carry a co-author trailer naming the model:

  ```
  Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
  ```

  Name the model that assisted (e.g. `Claude Opus 4.8 (1M context)`).
  Human-only commits omit the trailer.

## Granularity

- One logical change per commit; mechanical moves (`git mv`) separated
  from edits where practical so rename detection preserves blame.
- Version bumps are their own `chore: :bookmark: bump version to X.Y.Z`
  commit; example/notebook reruns their own
  `docs(example): :sparkles: rerun Jupyter Notebook for version X.Y.Z`.
