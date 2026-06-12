---
type: entity
title: "PathOfBuilding-PoE2 (PoB2) and CraftPath integration points"
tags: [mcp, integration]
sources: [mcp-chat-usecases-request]
url: https://github.com/PathOfBuildingCommunity/PathOfBuilding-PoE2
created: 2026-06-12
updated: 2026-06-12
---

# PathOfBuilding-PoE2 (PoB2)

Community-maintained offline build planner for Path of Exile 2 and the obvious
cross-reference for making CraftPath answers *build-aware* (use cases R7 and
P6 in [[mcp-chat-usecases]]). Status as checked 2026-06-12: MIT-licensed Lua
desktop app, v0.20.0 (June 2026), actively developed (1.7k stars, 379 forks).

Relevant capabilities: build share codes (base64+zlib XML), in-game clipboard
item import, full DPS/EHP calculation engine, trade-site query generation,
unique/rare template database derived from game data. **No formal API and no
documented headless mode in the fork's README**; upstream PoB1 ships
`HeadlessWrapper.lua`, parity to be verified at implementation time.

## Integration discussion points

Ordered by effort/risk; tool names refer to [[mcp-capability-roadmap]].

1. **Build-code import** (`import_pob_build`, low effort): decode the share
   code into class, level, equipped items and skills. Seeds meta queries
   ("good for *my* character"), item comparisons (B1) and affix
   recommendations (R7). The format is stable XML; no PoB runtime needed.
2. **Stat-weight scoring** (medium effort, recommended first): derive per-mod
   DPS/EHP weights from PoB once per build, cache them, and use them as the
   desirability function when ranking reachability outcomes and
   `recommend_affixes` suggestions. Turns "good enchant" from
   popularity-based into build-aware without running PoB in the server.
3. **DPS oracle** (high effort/risk, stretch): run PoB2's calc engine
   headlessly to score candidate outcome templates by true DPS/EHP delta
   (P6). Requires a Lua runtime in the server, verification of
   `HeadlessWrapper` parity in the fork, and tolerance for early-access data
   churn. Gate behind stat weights proving insufficient.
4. **Item text round-trip** (low effort): emit crafted target items as
   PoB-pasteable item text so players can A/B them in their build; reuse
   PoB2's trade-query generation logic for `get_item_price_estimate`.
5. **Mod-DB cross-validation** (low effort, ongoing value): PoB2's
   game-data-derived mod database as a second source to validate Craft of
   Exile affix data, whose tiers/weights drift across patches.

## Caveats

Desktop-app architecture with no API contract: integration means vendoring
data files or embedding code, which the MIT license permits. Pin to tagged
releases and re-validate after PoE2 patches.
