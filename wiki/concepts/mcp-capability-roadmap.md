---
type: concept
title: "MCP capability roadmap: tool surface, engine gaps, phasing"
tags: [mcp, roadmap, architecture]
sources: [mcp-chat-usecases-request]
created: 2026-06-12
updated: 2026-06-12
---

# MCP capability roadmap: tool surface, engine gaps, phasing

The consolidated engineering answer to [[mcp-chat-usecases]]: which MCP tools
the chat use-cases need, which engine capabilities are missing, where the data
comes from, and in what order to build it. Engine internals are documented in
[[architecture]]; the happy-path constraint and extension points in
[[development-strategy]].

Design principle: **the chat model does language and orchestration; the server
does deterministic math, ID resolution and caching.** Every tool is
JSON-in/JSON-out and composable. The one formal-language exception, the
EPPSSA craft-spec, is parsed server-side so a spec means the same thing every
time; the LLM never freehand-expands it.

## MCP tool surface

Today (`backend/crates/craftpath-server/src/mcp/mod.rs`): `submit_calculation`
(exact start/target snapshots), `get_job_status`, `get_job_result`,
`cancel_job`, `list_presets`, backed by the Redis job queue.

| Tool | Kind | Status | Purpose (use cases) |
|------|------|--------|---------------------|
| `get_job_status` | sync | keep | state, queue position, progress (R12, R14) |
| `cancel_job` | sync | keep | cancel queued/running job |
| `list_presets` | sync | keep | enumerate builders/analyzers |
| `submit_calculation` | job | extend | `target: TargetSpec` = exact \| template \| craft_spec; optional `budget_divines`, `restrict_currencies` (R3, P1, P3) |
| `get_job_result` | sync | extend | `format="mermaid"` route graph; `format="json"` gains per-step metadata: affix counts, risk class, branch chances (R9, R10, B8, P4) |
| `await_job` | job | shipped 2026-06-12 | long-poll until state change or timeout; MCP progress notifications where supported (R13, R14) |
| `search_affixes` | sync | shipped 2026-06-12 | fuzzy name/description search over cached affix tables; filters: base, location, class, min ilvl (B1, B2, B7, R3, P1) |
| `get_base_items` | sync | shipped 2026-06-12 | resolve base/item-class names to ids, max affixes/sockets (R3) |
| `parse_item` | sync | shipped 2026-06-12 (CoE JSON; in-game text open) | CoE JSON (exists internally) + in-game clipboard text -> `ItemSnapshot` + sanity check (B1, R2, R7) |
| `parse_craft_spec` | sync | shipped 2026-06-12 | EPPSSA grammar -> target template + validation + fan-out estimate (P1) |
| `get_meta_items` | sync | shipped 2026-06-12 (curated static v1) | ranked "currently good" archetypes per item class / char class / level bracket, with `data_freshness` (B6, R1, P2) |
| `get_currency_prices` | sync | shipped 2026-06-12 | currency -> divines map + div/exalt/chaos rates (R5) |
| `get_legal_actions` | sync | shipped 2026-06-12 | which currencies apply to *this* item, what each does, risk class (B3, B8, R6) |
| `simulate_action` | sync | shipped 2026-06-12 | one-step outcome distribution for (item, currency) (B7, R4) |
| `recommend_affixes` | sync | new | compatibility-filtered affix suggestions for an item (R7) |
| `get_item_price_estimate` | sync | new, flagged | trade2 listing percentiles for a template; rate-limited (B5, P5) |
| `import_pob_build` | sync | shipped 2026-06-12 | decode PoB2 share code -> class, level, items (P6, see [[pathofbuilding-poe2]]) |
| `submit_reachability` | job | new | budget (divines or currency inventory) + candidate templates -> expected cost, p(success within budget), best route each (B4, R2, R8, P2, P5) |

## Engine and server gaps

Mapped to crates; one-way layering per [[architecture]].

- **(a) Fuzzy targets**: `TargetMatcher` trait + `TargetTemplate` (slots as
  any-of affix sets with tier bounds and class constraints), generalizing the
  exact-target proximity heuristic and the terminal check in
  `craftpath-core/src/features/matrix/happy_path/builder.rs`. Fan-out capped
  (~200 concrete terminals; parser reports the estimate). Property tests:
  an exact-target template must reproduce legacy behavior. Medium effort,
  medium risk.
- **(b) Budget annotations**: `expected_total_cost` and
  `p_success_within_budget = 1-(1-p)^floor(budget/cost_per_try)` as route
  statistics in `features/analysis`. Low effort; the cheapest win.
- **(c) Reachability**: v1 = evaluate 3-10 LLM-supplied candidate templates
  as parallel jobs on the existing queue (pure composition of (a)+(b)).
  v2 (stretch) = target-free exploration builder + forward value pass;
  state-explosion risk, do not block chat UX on it.
- **(d) Craft-spec parser**: `features/craftspec` shipped 2026-06-12 -
  candidate pools, fan-out estimate, exact `ItemSnapshot` for fully pinned
  specs.
- **(e) `simulate_action`**: shipped 2026-06-12 as `features/inspect` -
  target-free weighted-pool distribution (additive orbs, desecration,
  essences, annulment) independent of the happy-path propagators.
- **(f) In-game clipboard item parser** alongside the CoE importer.
- **(g) Wire types**: TargetSpec, reachability job, progress payloads in
  `craftpath-proto` + root `proto/` (buf).
- **(h) Risk classification**: shipped 2026-06-12 in
  `domain/currency_data.rs` (`CurrencyRiskClass`), surfaced by
  `get_legal_actions`.
- **(i) Mermaid route renderer** in `features/render`.
- **(j) Inventory-constrained reachability**: v1 divine-value conversion +
  restricted action alphabet; v2 exact step counts in the search.
- **(k) Quality/catalyst modeling**: not in the state space today; stretch.
- **(l) Worker progress instrumentation**: publish phase/wave/frontier/RAM/
  routes-so-far to Redis during matrix build and analysis; feeds
  `get_job_status`, `await_job` and the WS endpoint
  (`craftpath-server/src/rest/ws.rs`).

## EPPSSA craft-spec mini-DSL

Compact target notation, parsed server-side.

```ebnf
spec      = slot+ ;                 (* 1..6 slots, validated vs base max_affix *)
slot      = letter , { qualifier } ;
letter    = "E" | "P" | "S" | "A" ; (* essence | prefix | suffix | abyss/desecrated *)
qualifier = tier | binding | fracture ;
tier      = digit+ , [ "x" ] ;      (* "P1" = prefix tier 1 or better; "x" = exact *)
binding   = "[" , ident , "]" ;     (* pin a specific affix: P[phys%] or P[#1234] *)
fracture  = "!" ;                   (* affix must be fractured *)
```

Case-insensitive; whitespace ignored. Examples: `EPPSSA` (one essence affix,
two open prefixes, two open suffixes, one desecrated mod, any tiers);
`E1P1P2S1SA` (tier bounds); `P[phys%]1 P[+levels] S[atk_speed]1 S S A`
(pinned slots).

Mapping: `E` -> any-of over the essence table for the base (pinned bindings
verified against `lookup_affix_essences`); `P`/`S` -> base prefix/suffix pools;
`A` -> desecrated pool for the base group; tier digits -> the engine's
existing minimum/exact tier bounds. The parser returns
`estimated_concrete_targets` (product of any-of cardinalities after
exclusive-group dedup) so the client pins slots before submitting an
explosive job; slot counts beyond the base's limits are hard errors.

## Data layer

New pluggable `MetaProvider` next to the existing CoE/poe.ninja adapters in
`features/data`, reusing the cached-HTTP plumbing (stale cache as fallback is
already the providers' behavior). Source priority:

1. poe.ninja PoE2 builds API, if available (verify at implementation time;
   undocumented, shape may change).
2. Official ladder + character API (OAuth, registered user-agent, honor rate
   limits; slow refresh).
3. Official trade2 API as a demand proxy and for `get_item_price_estimate`
   (highest ToS sensitivity: rate-limited token bucket, 15 min cache per
   template hash, feature-flagged, degrade to "unavailable" rather than queue).
4. **Curated static archetype JSON, always shipped**: per league/class/slot,
   top archetypes as craft-spec strings. This is the v1 implementation of
   `get_meta_items` and the permanent fallback.

Aggregates refresh every 6-24 h via a scheduled job on the existing queue;
every `get_meta_items` response carries `data_freshness` and `source` so the
chat model can disclose staleness.

## Client AI strategy: no second or local model required

Recommendation: **do not train a model and do not add a dedicated second AI.**

- Any tool-calling chat model orchestrates this server over MCP: Claude
  (native), ChatGPT (MCP connector support; alternatively the same surface as
  OpenAPI actions for custom GPTs), or local models via MCP-capable runtimes.
- Fine-tuning would bake in meta, prices and patch mechanics that churn
  weekly to monthly - exactly the data this design keeps behind refreshable
  tools. A trained model goes stale; a tool call does not.
- The classic "second AI" need, fuzzy affix-name resolution, is plain string
  matching inside `search_affixes`; no embedding model needed.
- A local LLM is a privacy/cost deployment option, not a requirement. Answer
  quality scales with the client model's tool-use ability, so every tool must
  be self-describing: precise descriptions, typed parameters, actionable
  error messages (this is the real "model interface" work).

## Phasing

| Phase | Scope | Unblocks |
|-------|-------|----------|
| 1 (done 2026-06-12) | sync lookups (`search_affixes`, `get_base_items`, `get_currency_prices`), `get_legal_actions` + risk table, `parse_item` (CoE), static `get_meta_items` | B1-B3, B6-B8, R1, R5, R6, R12; R7 partial; no engine changes |
| 2 (partial: parser, `simulate_action`, `await_job` + PoB import done 2026-06-12) | `parse_craft_spec`, budget annotations (b), `simulate_action`, step metadata + Mermaid in `get_job_result`, `await_job` + progress instrumentation (l) | R4, R9, R10, R13, R14; P1 for fully pinned specs |
| 3 | fuzzy targets (a) through builder, proto and `submit_calculation` | R3, P1 complete; P3, P4 |
| 4 | `submit_reachability` (c v1, inventory v1), live meta ingestion, `get_item_price_estimate`, `import_pob_build` + stat weights | B4, B5, R2, R8; R7 complete; P2, P5; P6 v1 |
| 5 | stretch: reachability v2, inventory exact step counts, quality modeling (k), PoB DPS oracle | R11, P6 full |

Phases 1 and 2 are independent; phase 4 needs 2+3. Each phase is a standalone
implementation task against `backend/crates/`.
