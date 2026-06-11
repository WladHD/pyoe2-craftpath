[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/WladHD/pyoe2-craftpath/release_python.yml?branch=main)](https://github.com/WladHD/pyoe2-craftpath/actions/workflows/release_python.yml)
[![GitHub Repo](https://img.shields.io/badge/GitHub-pyoe2--craftpath-blue?logo=github)](https://github.com/WladHD/pyoe2-craftpath)
[![PyPI - Version](https://img.shields.io/pypi/v/pyoe2-craftpath?logo=pypi)](https://pypi.org/project/pyoe2-craftpath/)
[![Crates.io Version](https://img.shields.io/crates/v/pyoe2-craftpath?logo=rust)](https://crates.io/crates/pyoe2-craftpath)
[![GitHub License](https://img.shields.io/github/license/WladHD/pyoe2-craftpath)](https://github.com/WladHD/pyoe2-craftpath/blob/main/LICENSE)

# *Py*oE 2 - CraftPath
A tool for Path of Exile 2 to find the best craftpaths based on the categories: *most likely, most efficient and cheapest*, between a starting item and a target item.

Available as Python package [`pyoe2-craftpath`](https://pypi.org/project/pyoe2-craftpath/) or as "bro just gimme something that goes brrr"-executable command-line utility for Windows under [Releases](https://github.com/WladHD/pyoe2-craftpath/releases). Bindings for Python are generated with [PyO3](https://github.com/PyO3/pyo3), to let you build your own data analysis pipeline upon the calculated items and craftpaths. Made possible by the power of [*🦀 Rust*](https://www.reddit.com/r/linuxmemes/comments/1b7y5vv/rust/).

Built and tested for Path of Exile 2 on version `0.4.0`. Supported Python versions and platforms are determined by the [automated pipeline, here](https://github.com/WladHD/pyoe2-craftpath/actions/workflows/release_python.yml) (should support all widely used platforms and versions `>=3.10`).

The primary goal is to provide a framework for calculating craft paths and to enable its integration into native applications (over FFI), including overlays or mobile apps ... or just run easily as a [Windows executable](https://github.com/WladHD/pyoe2-craftpath/releases) or [Python library](https://pypi.org/project/pyoe2-craftpath/).

## About
To keep it short, I was introduced to Path of Exile 2 and enjoyed it quite a bit.
After reaching higher levels and starting to get the hang of things, I became interested in crafting.
As big noob, I was completly overwhelmed with the information available.

**Me need simple. Me want good item. How get good item?**

*CraftPath*. The purpose of this tool is give it information about your current item and the affixes you want it to have, then let it efficiently calculate possible craftpaths; Without the need to manually look up mod weights, mod groups, or spend hours crunching probabilities on a Casio calculator, as all true PoE gamers do[^1]. It simulates all *sensible* currency sequences that can be applied on a starting item, and collects the best routes that lead to the given target item, based on the specified statistic (more in [Development Strategy and Caveats](#strategy-and-development)).

## 🚧 Notice for Versions Below 1.0.0
Keep in mind that this project is in its early stages, and can contain bugs and lack features. Therefore, the section [Features](#features) should give an overview over *all* planned/completed/unplanned currencies and known bugs. If your topic is not documented there, it is yet unknown and not reviewed. Feel free to create an [Issue](https://github.com/WladHD/pyoe2-craftpath/issues) with more information!

My plan is of course to reach version `1.0.0` ... which depends on the traction this project gains, which in turn affects how much free time I'm motivated to dedicate to it, which in turn leads to a more robust and well-rounded project. Until then, this notice lingers... possibly forever. 🧙‍♂️

## Features<a name="features"></a>
| **Currency**                  | **Options**                                                                     | **Status**                             | **Note**                                                                                                                                                                                                                                |
| ----------------------------- | ------------------------------------------------------------------------------- | -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Orb of Transmutation**      | Normal, Greater (44), Perfect (70)                                              | Completed                              |                                                                                                                                                                                                                                         |
| **Orb of Augmentation**       | Normal, Greater (44), Perfect (70)                                              | Completed                              |                                                                                                                                                                                                                                         |
| **Regal Orb**                 | Normal, Greater (35), Perfect (50), Homogenising Coronation[^3]                 | Completed                              |                                                                                                                                                                                                                                         |
| **Orb of Alchemy**            |                                                                                 | Planned                                | Action exists in the CoE emulator vocabulary; see the coverage TODO in `backend/README.md`.                                                                                                                                             |
| **Chaos Orb**                 | Normal, Greater (35), Perfect (50), Dex/Sin Erasure, *Whittling                 | Completed                              | Whittling removes the affix based on minimal item level, not tier.                                                                                                                                                                      |
| **Exalted Orb**               | Normal, Greater (35), Perfect (50), Dex/Sin Exaltation, Greater Exaltation, Homogenising Exaltation[^3] | Completed             | `Omen of Greater Exaltation`: adds two affixes at once, exact pair probability without replacement.                                                                                                                                |
| **Orb of Annulment**          | Dex/Sin Annulment, Omen of Light                                                | Completed                              | `Omen of Light`: the next Annulment removes only Desecrated modifiers, enabling the "temporary blocker" workflow.                                                                                                                                        |
| **Divine Orb**                |                                                                                 | Planned                                | Per-tier value ranges are already parsed from CoE data; see the coverage TODO in `backend/README.md`.                                                                                                                                   |
| **Artificers Orb**            |                                                                                 | Completed                              | Click item. Socket. Much wow. Such  awesome.                                                                                                                                                                                            |
| **Fracturing Orb**            |                                                                                 | Completed                              | Algorithm respects fractured affixes if present on the start item. Also fractures an affix if 4 are present.                                                                                                                            |
| **Auto. Fracturing Orb**      |                                                                                 | Planned                                | A flag is planned, that adds variations of the target item with one fractured affix. This should give insight of which affixes are the most valuable if fractured.                                                                      |
| **Vaal Orb**                  | Omen of Corruption[^4]                                                          | Completed                              | Models the socket branch *and* corruption implicits (weighted from CoE data); targets may include one implicit. The mod-reroll branch is never target-approaching and stays unmodeled.            |
| **Lesser to Greater Essence** |                                                                                 | Completed                              |                                                                                                                                                                                                                                         |
| **Perfect Essence**           | Dex/Sin Crystallisation                                                         | Completed                              | Algorithm tries to create a temporary affix to swap with, if otherwise unreachable.                                                                                                                                                     |
| **Alloys**                    | Dex/Sin Crystallisation                                                         | Completed                              | Alloys (e.g. `Transcendent Alloy`) apply like Perfect Essences (Rare-only, remove-one-add-one). No poe.ninja price yet; costs fall back to a 1-Exalted placeholder.                                                     |
| **Desecration**               | Abyssal Echoes, Blackblooded, Liege, Sovereign, Sin/Dex Necromancy              | Partially Completed, Not Planned       | "Blackblooded, Liege, Sovereign" are forced. Loose propagation of affixes is not planned. **ATTENTION** Desecration weights are unknown and are treated equally by the algorithm; all desecration weights = 1.                           |
| **Others**                    |                                                                                 | On request                             | The full coverage matrix (implementable now vs. waiting for CoE data) lives in [`backend/README.md`](backend/README.md). [Open an issue](https://github.com/WladHD/pyoe2-craftpath/issues) if something is missing.                      |

All affix weights are fetched from [`craftofexile.com`](https://www.craftofexile.com/weightings); refer to the given link for infos about how the weights are collected. Game mechanics are verified against the CoE emulator, [poe2wiki](https://www.poe2wiki.net) and the poe.ninja API - see [`MECHANICS.md`](backend/crates/craftpath-core/MECHANICS.md).

## How To Run<a name="how-to"></a>
Moved to the wiki: [How to run CraftPath](wiki/concepts/how-to-run.md) - Python library (local or against a backend), the CLI (`pyoe2-backend cli` / the Windows executable, incl. the [video walkthrough](https://www.youtube.com/watch?v=27J1Kjs8q5E)), the self-hosted REST/MCP backend, and the Rust crate.

## How It Works<a name="strategy-and-development"></a>
A craft calculation runs in two stages. First, a **matrix builder** simulates every *sensible* currency application from your starting item (the "happy path": only steps that gain a wanted affix or shed an unwanted one), producing a graph of item states with exact per-step probabilities from CraftOfExile's weight tables and prices from poe.ninja. Second, a **route engine** searches that graph for the best routes per statistic - highest chance, cheapest, most efficient (cost × tries needed for a 60% success) - using exact K-best graph search (Yen's algorithm and a bi-criteria Pareto search) instead of brute-force enumeration, so even 6-affix targets resolve in milliseconds. Results come back as ranked routes with step-by-step currencies, chances and costs, identically through Python, the CLI, the REST backend or MCP.

```mermaid
flowchart LR
    A["Start + target item"] --> B["Matrix builder<br/>happy-path simulation"]
    B --> C["Graph of item states<br/>exact chances + costs"]
    C --> D["Route engine<br/>K-best search per statistic"]
    D --> E["Ranked routes<br/>chance / cost / efficiency"]
```

The deep dives live in the wiki:
- [Architecture: how CraftPath works](wiki/concepts/architecture.md) - data flow, matrix building, the K-best engine, the four surfaces
- [Development strategy and caveats](wiki/concepts/development-strategy.md) - the happy-path constraint, known edge cases (temporary essence steps, desecration weights), and the route-engine internals

## Contribution / Dev Usage
Moved to the wiki: see the [contribution section of the development strategy](wiki/concepts/development-strategy.md#contributing--dev-usage) and the [commit conventions](wiki/concepts/commit-conventions.md). Short version: PRs welcome, conventional-commits format, preferably with a test.

## Acknowledgments
- Of course, [Grinding Gear Games](https://www.grindinggear.com/) for providing Path of Exile 2, that got me hooked to the extent of actually coding this.

- [CraftOfExile](https://www.craftofexile.com/) that permitted me to use their [item data](https://www.craftofexile.com/json/poe2/main/poec_data.json). **CraftPath would not be possible without it.** CoE offers an extensive, crunched mapping for weights, items, affixes, etc. Moreover I integrated CoE's Emulator Export outputs to parse the starting/target item, offering an external, easy capture of item information over a GUI. Since I as noob needed something hands-on, easy to use, CraftOfExile was essential for this project.

- [poe.ninja](https://poe.ninja/) for providing a public API to fetch up-to-date currency exchange prices. Used by CraftPath to calculate the cost of a crafting path, and subsequently corresponding cost-based analysis. Cudos for hosting and keeping a free, public API alive for such a long time!


## Disclaimer
**CraftPath is not affiliated with or endorsed by Grinding Gear Games**

## License
[MIT License](https://github.com/WladHD/pyoe2-craftpath/blob/main/LICENSE)

[^1]: Source: trust me, bro

[^2]: Actually item state ([ItemSnapshot](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/api/item.rs)), but in the given example w/e. The item state contains more information like rarity, base item id, level, etc. 

[^3]: Drop-disabled since game patch `0.4.0` (legacy items still work). Modeled, and excludable via `CalculationConfig::legacy_currencies()`.

[^4]: Unobtainable since game patch `0.5.0` (legacy items only). Modeled, and excludable via `CalculationConfig::legacy_currencies()`.