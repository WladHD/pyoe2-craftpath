# Game-patch 0.5.0 mechanics — verification table

Verified 2026-06-11 against (in priority order): the CraftOfExile PoE2 emulator
code (`poe2.js` config/omen/desecration data + `package.js` simulation
handlers), [poe2wiki](https://www.poe2wiki.net), and the poe.ninja PoE2
exchange API (league "Runes of Aldur" = the 0.5.0 league). Code comments cite
rows as `MECHANICS.md V<n>`.

| ID | Mechanic | Verified finding | Confidence |
|----|----------|------------------|------------|
| V1 | Vaal Orb outcomes | 4 uniform branches: add socket / add corruption implicit / partial mod reroll ("brick") / no change — 25% each, mutually exclusive. No rarity-reroll branch. On jewellery the socket branch acts as "no change". **Omen of Corruption** removes the "no change" branch (1/3 each) but **is unobtainable since 0.5.0** (legacy items only; absent from poe.ninja league listings). CoE emulator divergence: its "brick" is a full 6-mod reroll while game text says "reroll up to three modifiers" — irrelevant for us (never target-approaching). | High |
| V2 | Corruption implicit pick | Weight-proportional single draw from the base's corrupted-mod pool (101 mods in CoE data). All tier weights are 1 → effectively uniform over the ~13-14 implicits per base. Exactly one implicit per corruption. True in-game weighting (possibly ilvl/base dependent) unverified. | High (CoE), Medium (in-game weights) |
| V3 | Socket + implicit in one Vaal | Impossible — a single `switch` picks exactly one branch. Targets wanting both are rejected with a warning. | High |
| V4 | Omen of Greater Exaltation | Next Exalted adds 2 mods, drawn **without replacement** (2nd roll excludes the 1st mod's modgroup). Combinable with Dextral/Sinistral Exaltation (both mods on the forced side; dextral+sinistral together is illegal). Also combinable with Homogenising (each mod matches independently) — **not modeled in v1** (homogenising is legacy-only anyway). If two mods cannot both be generated the craft fails. | High |
| V5 | Omen of Light | **An ANNULMENT omen**: the next Orb of Annulment removes only Desecrated modifiers (regardless of exclusivity). Implemented in `orb_of_annulment.rs`. It has nothing to do with desecration-choice rerolls (the original article's claim was wrong). Stacks with Dextral/Sinistral Annulment in-game (not modeled in v1 — niche). | High |
| V6 | Desecration choice | Well of Souls reveal offers a choice of **3 modifiers**; number of desecrated-exclusive options is 1/2/3 with rates [0.8, 0.15, 0.05], rest filled from the regular pool. **Omen of Abyssal Echoes = reroll the 3-option set once** → modeled as k=2 pick-sets (baseline k=1). Our `hit_chance_at_least_once(W, w, k)` approximates each set as one weighted draw — a simplification (3 options per set are not modeled). Max ONE desecrated mod per item; desecrating a full item removes a random mod first. | High (semantics), Medium (our approximation) |
| V7 | Alloys | Behave exactly like Perfect essences: **Rare-only**, always remove-one-then-add-one (even when not full), one guaranteed mod. Not usable on Magic items. Loose (omen-less) use is normal. Dextral/Sinistral **Crystallisation force the REMOVED side** (suffix/prefix respectively) and per the wiki DO apply to alloys (CoE's emulator only applies them to perfect essences — wiki followed). 13 alloys in the 0.5.0 data (Adaptive…Swift, The Runebinder's/Runefather's, Transcendent). | High; Medium-High (omens-on-alloys) |
| V8 | Homogenising omens | Candidate mod must share **at least one** tag with the **union** of all existing mods' tags (any-match, weighted normally) — exactly what `homogenized_mods` + the disjoint check implement. CoE excludes only tag 38 ("Drop") from the union (now mirrored in `Item::build_with`); desecration tags 39-41 never occur on exalt-pool mods. **Both Homogenising omens are drop-disabled since 0.4.0** (legacy items only, no 0.5.0-league price) — re-enabled in the model per user decision; flip `HOMOGEN_OMEN_GROUP` to disable. | High |
| V9 | Desecrated mods as blockers | Removable by Chaos/Annulment (only fractured mods are protected). Omen of Light exists precisely for targeted removal. Unrevealed desecrated mods occupy their slot; revealed ones can even be fractured. | High |
| V10 | poe.ninja price names | type=Ritual carries all omens incl. "Omen of Greater Exaltation" and "Omen of Light" (NOT Homogenising/Corruption — removed). type=Abyss carries the bones. **Alloys are not on the exchange API at all** → price lookups fall back to the 1-Exalted placeholder; alloy-route costs are unreliable until a price source exists (official trade API or user-supplied). | High |

## Known model simplifications (v1)

- Greater Exaltation enumerates only pairs of missing **target** affixes; the
  "one wanted + one random" outcome is dropped (proximity-neutral under
  happy-path semantics) → Greater-Exalt route chances are slightly
  underestimated.
- Greater Exaltation × Homogenising combination not enumerated.
- Omen of Light × Dextral/Sinistral Annulment combination not enumerated.
- The desecration 3-option choice is approximated as one weighted draw per
  pick-set (see V6).
- Omen of Corruption and Homogenising omens are modeled but unobtainable in
  the 0.5.0 league — their routes only make sense with legacy stock, and
  their prices fall back to the 1-Exalted placeholder.
