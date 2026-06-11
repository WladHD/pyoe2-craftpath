---
type: concept
title: "Development strategy and caveats"
tags: [algorithm, contribution]
sources: []
created: 2026-06-11
updated: 2026-06-11
---

# Development strategy and caveats

The inner workings of the calculation, its deliberate constraints, and how to
extend or contribute. Replaces (and updates) the old README sections
"Development Strategy and Caveats" and "Contribution / Dev Usage". For the
full system walkthrough see [[architecture]].

## Two-part architecture

Returning the best paths for custom statistics splits into two parts:

- A **matrix builder** (`MatrixBuilder` trait,
  `backend/crates/craftpath-core/src/api/calculator.rs`; propagators behind
  the `MatrixPropagator` trait) collects *all sensible* item states with
  their possible *next* states, the currencies applied and exact chances.
  "*All sensible*" is the implementation's choice - the shipped
  `HappyPathMatrixBuilder`
  (`backend/crates/craftpath-core/src/features/matrix/happy_path/builder.rs`)
  is described below. The result is a graph of item snapshots keyed by hash.
- **Statistic analyzers** search that graph for the best routes. Two trait
  families exist: `StatisticAnalyzerPaths` (best *unique routes*) and
  `StatisticAnalyzerCurrencyGroups` (best *currency sequences*).
  Implementations live under
  `backend/crates/craftpath-core/src/features/analysis/`.

## The Happy Path constraint

To bound the state space, the builder stays on the *happy path*: additive
currencies (e.g. `Exalted Orb`) only roll affixes from the desired target
set; subtractive currencies (e.g. `Orb of Annulment`) only remove unwanted
affixes. Simply put: if the algorithm were a player, it would immediately
stop crafting an item *that does not gain a wanted affix (or lose an
unwanted one)*.

This may miss routes that require temporarily applying an undesired affix.
Known edge case: applying a `Perfect Essence` (or 0.5.0 Alloy) to a full
affix side would remove a wanted affix, so the happy path stops. The
`PerfectEssencePropagator` therefore inserts a deliberate off-path
*temporary* affix first (forced via `Dextral`/`Sinistral Crystallisation`),
which the essence then replaces. More such edge cases likely exist - they
need explicit propagator support; please open an issue if you find one.

Caveat that still stands: **desecration weights are unknown** - CoE carries
no weights for desecrated mods, so the algorithm treats them all as weight 1.

## Route analysis: exact K-best search (since 2026-06)

The original analyzer enumerated *every* simple path by DFS, filtering
cycles per edge and recomputing weights per step - usable, but deep targets
(6 affixes+) meant millions of *senseless* checks and gigabytes of route
storage. That implementation is gone.

The current engine
(`backend/crates/craftpath-core/src/features/analysis/engine/`) treats the
matrix as a weighted digraph and computes the top-K routes *exactly*:

- **chance** (product of edge chances) and **cost** (sum of edge prices):
  Yen's K best loopless paths over a metric-monotone Dijkstra,
  O(K·V·(E + V log V));
- **efficiency** (`cost × tries-for-60%(chance)` - not edge-decomposable):
  bi-criteria label-setting search in ascending-cost order with per-node
  K-dominance Pareto pruning; cycles prune themselves because every cycle
  strictly increases cost.

Equivalence to exhaustive enumeration is pinned by
`tests/test_fast_engine_equivalence.rs` against the all-path oracle preset
(`UniquePathChanceMemoryHeavy`), which remains available as the
"give me everything" mode. Measured: ~224× faster on a 4-affix fixture and
milliseconds on a 6-affix fixture where exhaustive enumeration aborts at
>4 GB RAM. The mechanics themselves are verified per
`backend/crates/craftpath-core/MECHANICS.md`.

## Contributing / dev usage

Published on [crates.io](https://crates.io/crates/pyoe2-craftpath) and
[PyPI](https://pypi.org/project/pyoe2-craftpath/). Build your own extension
as a Rust crate depending on `craftpath-core`, or via Python. For FFI to
another language, open an
[issue](https://github.com/WladHD/pyoe2-craftpath/issues).

Pull requests are welcome; requirements are the
[[commit-conventions]] and preferably a test for new code.

The central extension points are the preset enums -
`MatrixBuilderPreset`, `StatisticAnalyzerPathPreset`,
`StatisticAnalyzerCurrencyGroupPreset`
(`backend/crates/craftpath-core/src/features/{matrix,analysis}/presets/`) -
which integrate implementations into both the Rust and Python surfaces
(e.g. a `DynMatrixBuilder` can be passed as an argument in the Jupyter
example, sections 5-6). For configuration without new code (disabling legacy
omens, limits), use `CraftSession` + `CalculationConfig`
(`backend/crates/craftpath-core/src/api/session.rs`).

## Where this fits

- [[architecture]] - the full data-flow walkthrough
- [[how-to-run]] - running the tool
- [[commit-conventions]] - contribution format
