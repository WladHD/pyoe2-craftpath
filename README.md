<!---
GitHub Actions Workflow Status, Version, Version Crates, Version PyPi, License
-->
# *Py*oE 2 - CraftPath
A tool for Path of Exile 2 to find the best craftpaths based on the categories: *most likely, most efficient and cheapest*, between a starting item and a target item.

Available as Python package [`pyoe2-craftpath`](https://pypi.org/project/pyoe2-craftpath/) or as "bro just gimme something that goes brrr"-executable command-line utility for Windows under [Releases](https://github.com/WladHD/pyoe2-craftpath/releases). Bindings for Python are generated with [PyO3](https://github.com/PyO3/pyo3), to let you build your own data analysis pipeline upon the calculated items and craftpaths. Made possible by the power of [*🦀 Rust*](https://www.reddit.com/r/linuxmemes/comments/1b7y5vv/rust/).

Build and tested for Path of Exile 2 on version `0.3.1` and Python `3.12.12`.

## About
To keep it short, I was introduced to Path of Exile 2 and enjoyed it quite a bit.
After reaching higher levels and starting to get the hang of things, I became interested in crafting.
As big noob, I was completly overwhelmed with the information available.

**Me need simple. Me want good item. How get good item?**

*CraftPath*. The purpose of this tool is give it information about your current item and the affixes you want it to have, then let it efficiently calculate possible craftpaths; Without the need to manually look up mod weights, mod groups, or spend hours crunching probabilities on a Casio calculator, as all true PoE gamers do[^1]. It simulates all *sensible* currency sequences that can be applied on a starting item, and collects the best routes that lead to the given target item, based on the specified statistic (more in [Development Strategy and Caveats](#strategy-and-development)).

## 🚧 Notice for Versions Below 1.0.0
Keep in mind that this project is in its early stages, and can contain bugs and lack features. [Features](#features) should contain an overview over planned/completed/unplanned currencies and known bugs. If your topic is not documented there, it is yet unknown and not reviewed. Feel free to create an [Issue](https://github.com/WladHD/pyoe2-craftpath/issues) with more information!

My plan is of course to reach version `1.0.0` ... which depends on the traction this project gains, which in turn affects how much free time I'm motivated to dedicate to it, which in turn leads to a more robust and well-rounded project. Until then, this notice lingers... possibly forever. 🧙‍♂️

## Features<a name="features"></a>
| **Currency**                  | **Options**                                                                     | **Status**                       | **Note**                                                                                                                                                                                                                                |
| ----------------------------- | ------------------------------------------------------------------------------- | -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Orb of Transmutation**      | Normal, Greater (55), Perfect (70)                                              | Completed                        |                                                                                                                                                                                                                                         |
| **Orb of Augmentation**       | Normal, Greater (55), Perfect (70)                                              | Completed                        |                                                                                                                                                                                                                                         |
| **Regal Orb**                 | Normal, Greater (35), Perfect (50), Homogenising Coronation                     | Completed                        |                                                                                                                                                                                                                                         |
| **Orb of Alchemy**            |                                                                                 | Not Planned                      | Too random to craft deterministically.                                                                                                                                                                                                  |
| **Chaos Orb**                 | Normal, Greater (35), Perfect (50), Dex/Sin Erasure, *Whittling                 | Completed                        | Whittling removes the affix based on minimal item level, not tier.                                                                                                                                                                      |
| **Exalted Orb**               | Normal, Greater (35), Perfect (50), Dex/Sin Exaltation, Homogenising Exaltation | Completed                        |                                                                                                                                                                                                                                         |
| **Orb of Annulment**          | Dex/Sin Annulment                                                               | Completed                        |                                                                                                                                                                                                                                         |
| **Divine Orb**                |                                                                                 | Not Planned                      | Different use-case.                                                                                                                                                                                                                     |
| **Artificers Orb**            |                                                                                 | Planned                          |                                                                                                                                                                                                                                         |
| **Fracturing Orb**            |                                                                                 | Partially Completed, Planned     | Algorithm respects fractured affixes if present on the start item; does not create fractured affixes automatically yet.                                                                                                                 |
| **Vaal Orb**                  |                                                                                 | Planned                          |                                                                                                                                                                                                                                         |
| **Lesser to Greater Essence** |                                                                                 | Completed                        |                                                                                                                                                                                                                                         |
| **Perfect Essence**           | Dex/Sin Crystallisation                                                         | Completed                        | Algorithm tries to create a temporary affix to swap with, if otherwise unreachable.                                                                                                                                                     |
| **Desecration**               | Abyssal Echoes, Blackhooded, Liege, Sovereign, Sin/Dex Necromancy               | Partially Completed, Not Planned | "Blackhooded, Liege, Sovereign" are forced. Loose propagation of affixes is not planned. **ATTENTION** Desecration weights are unknown and are treated equally by the algorithm; all desecration weights = 1.                           |
| **Others**                    |                                                                                 | On request                       | If not explicitly listed in this table, other crafting methods have not been reviewed yet or are not planned. [Open an issue](https://github.com/WladHD/pyoe2-craftpath/issues) if I forgot something that you would find nice to have. |

## How To Run<a name="how-to"></a>
This tool is actually intended to be used in Python. Refer to the [extended Python example as Jupyter Notebook](https://github.com/WladHD/pyoe2-craftpath/blob/main/python_examples/example_calculator_for_example_items.ipynb) or just skim through the [`python_examples`](https://github.com/WladHD/pyoe2-craftpath/tree/main/python_examples) directory for commented usage in Python.

The following section shows a guide for the *quick-n-dirty* approach to run the Windows executable via the console, since I wanted to offer a simple(r) solution for those who just want to have a basic overview and are not planning to create further analytical pipelines. 

First things first. Download the program from [Releases](https://github.com/WladHD/pyoe2-craftpath/releases).

```bash
pyoe2-craftpath.exe [options]
```

Available optional, options:
<details>
<summary><code>--start_item_path &lt;Path to JSON File&gt;</code></summary>

**Default:** `pyoe2-craftpath/startitem.json`  
Provides the file location of the saved item to treat as the starting item of the craft.  
Use [CraftOfExile](https://www.craftofexile.com/?game=poe2) → Emulator → Export and paste the output into `pyoe2-craftpath/startitem.json`.
</details>

<details>
<summary><code>--target_item_path &lt;Path to JSON File&gt;</code></summary>

**Default:** `pyoe2-craftpath/targetitem.json`  
Provides the file location of the saved item to treat as the end item of the craft.  
Use [CraftOfExile](https://www.craftofexile.com/?game=poe2) → Emulator → Export and paste the output into `pyoe2-craftpath/targetitem.json`.
</details>

<details>
<summary><code>--cache_path &lt;Path to Temp Folder&gt;</code></summary>

**Default:** `pyoe2-craftpath`  
Used for caching [CraftOfExile's](https://www.craftofexile.com/?game=poe2) and  
[PoE.Ninja's](https://poe.ninja/poe2/economy/) datasets.  
**The folder must already exist.**  
Mostly to express consent for the program to cache things and edit that folder’s contents.
</details>

<details>
<summary><code>--poe2_league &lt;League&gt;</code></summary>

**Default:** `Rise of the Abyssal`  
Fetches [PoE.Ninja's](https://poe.ninja/poe2/economy/) economy data for the specified league.
</details>

<details>
<summary><code>--amount_routes &lt;Number&gt;</code></summary>

**Default:** `5`  
Number of craft paths collected and printed per stats category.  
(Current categories: highest chance, most efficient, cheapest → `3 × amount_routes` shown.)
</details>

<details>
<summary><code>--no_updates</code></summary>

**Default:** checks GitHub for updates  
If set, CraftPath will **not** query GitHub or check for newer versions.
</details>

<details>
<summary><code>--no_groups</code></summary>

**Default:** collects all possible paths  
If set, CraftPath will **not** collect all possible path groups.  
This greatly reduces RAM usage but results in less complete output.
</details>

<details>
<summary><code>--max-ram &lt;&lt;Number&gt;[GB|KB|MB]&gt;</code></summary>

**Default:** `1GB`  
Sets the maximum amount of RAM the program may use during path collection.
</details>

## Development Strategy and Caveats<a name="strategy-and-development"></a>
The following section explains the inner workings of my algorithm, if you are interested in contributing or just want to have an idea of how this tool works. If you're only here for the crafting you can skip this. 

The architecture to return the best paths based on custom statistics consists of two important parts. Firstly, the [MatrixPropagator](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/api/matrix_propagator.rs) and secondly the [StatisticAnalyzer(s)](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/api/calculator.rs).
- A matrix propagator's job is to collect *all sensible* items with *all sensible* possible *next* items, specified currencies and their chances. The definition of *all sensible* is to be defined by the actual algorithm implementing the trait `MatrixBuilder`. This structure is efficiently constructed as a tree. For an actual implementation refer to [HappyPathMatrixBuilderImpl](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/matrix/happy_path_impl/happy_path_matrix_builder_impl.rs).
- A statistical analyzer now uses the constructed matrix to traverse all possible paths and calculates weights for each unique route. The weights are dependent on the algorithm and can be e. g. the chance, the cost, etc. Each analyzer can specify if lower is better. Currently two predefined general traits exist, firstly the [StatisticAnalyzerPaths](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/api/calculator.rs), which returns the best *unique routes*, and secondly [StatisticAnalyzerCurrencyGroups](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/api/calculator.rs), which returns the best *currency sequences*. Actual implementations are contained [here](https://github.com/WladHD/pyoe2-craftpath/tree/main/src/calc/statistics/analyzers).

### Matrix Builder Implementation
To constraint propagation and massivly reduce complexity, [my algorithm](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/matrix/happy_path_impl/happy_path_matrix_builder_impl.rs) tries to stay on the `Happy Path` as much as possible. That means, that affixes that can be rolled, but are not included in the desired *affix state*, will not be considered for additive currencies like [`Exalted Orb`](https://github.com/WladHD/poe2-craftpath/blob/main/poe2-craftpath/src/calc/propagators/exalted_orb.rs). Subtractive currencies like [`Orb of Annulment`](https://github.com/WladHD/poe2-craftpath/blob/main/poe2-craftpath/src/calc/propagators/orb_of_annulment.rs) will only result in *affix states*, that lose unwanted affixes. (= Definition of *all sensible*) **Simply put, if my algorithm was a player, it would immediatly stop crafting an item, *that does not gain an affix from the desired affixes (or lose an unwanted affix)***.

While this approach enables more efficient path construction, it may miss routes that can only be reached by temporarily applying an undesired affix. Such an edge case can be found by trying to apply `Perfect Essence`. 


> Let's assume we have an item with three desirable prefixes that we want to keep, and we plan to apply a `Perfect Essence` to add a suffix. Naivly applying it results in an item with two prefixes and the new suffix from the `Perfect Essence`. This action is not done by [HappyPathMatrixBuilderImpl](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/matrix/happy_path_impl/happy_path_matrix_builder_impl.rs), since it would remove a wanted affix. Thus, propagation stops and completes without finding a craft path that leads to the target item at all.

To fix this specific edge case, [PerfectEssencePropagator](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/matrix/happy_path_impl/propagators/perfect_essences.rs) introduces an additional temporary step, forcing propagation outside of the `Happy Path`: expanding on the mentioned example, it first applies a suffix from the unwanted affix pool. This ensures that the three desired prefixes remain untouched, while the temporary suffix can be replaced with the `Perfect Essence` and the `Dextral Crystallisation` omen.

I'm sure many more such edge cases exist, and those need to be specifically implemented. If you can think of any, please tell me :3

### Analysis Implementation
I haven't implemented anything crazy for this one, so I'll keep it short.
The most interesting detail is probably the need to filter out theoretical cyclic propagations. I'm not aware of this happening in my algorithm, but it is an edge-case that must be handled to avoid infinite loops. A theoretical possibility would be f. e. `Exalted Orb`, `Orb of Annulment`, `Exalted Orb`, `Orb of Annulment` ... resulting in the *same affix state*. Therefore [`calculate_crafting_paths`](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/statistics/analyzers/collectors/utils/statistic_analyzer_unique_collector.rs) only continues paths, that do not contain the same *affix state* twice[^2]. What would be nice as well, is an improved filtration of *senseless* routes ... but the definition for *senseless* routes is actually the hard task here. Cauz the above example only filters out *the same affixes*. It would still calculate the same currency sequence for different affixes, which is *senseless*, but I do not know how to filter it out *efficiently* yet. Hence, the current version only filters out cycles, and lets the [StatisticalAnalyzers](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/api/calculator.rs) handle the sorting. Since the *senseless* routes will have a much worse weight than the best ones, they will be filtered out naturally; on the expense of checking *senseless* routes, which really is a problem on *deep* paths (6 affixes+), resulting in *millions* of *senseless* checks.


## Contribution / Dev Usage
I've published the project on [`crates.io`](https://crates.io/) (and [`PyPI`](https://pypi.org/)). You can either use the [API](https://github.com/WladHD/pyoe2-craftpath/tree/main/src/api) to build your own extension as own Rust crate depending on `pyoe2-craftpath`.

If you want, you can also create a pull request to have it directly included here. The only requirement is the usage of the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format for your commit messages and preferably a new test for your code.

For both options, the most important things would be the central enums [StatisticAnalyzerCurrencyGroupPreset](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/statistics/presets/statistic_analyzer_currency_group_presets.rs), [StatisticAnalyzerPathPreset](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/statistics/presets/statistic_analyzer_path_presets.rs) and [MatrixBuilderPreset](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/matrix/presets/matrix_builder_presets.rs) which are probably the things that most likely could be offered by an extension. The mentioned enums provide specific, usuable implementations for both Rust and Python and integrate seamlessly into the rest of CraftPath's "ecosystem".

Feel free to open an [Issue](https://github.com/WladHD/pyoe2-craftpath/issues) to ask about technical stuff. 

## Acknowledgments
- Of course, [Grinding Gear Games](https://www.grindinggear.com/) for provinding Path of Exile 2, that got me hooked to the extent of actually coding this.

- [CraftOfExile](https://www.craftofexile.com/) that permitted me to use their [item data](https://www.craftofexile.com/json/poe2/main/poec_data.json). **CraftPath would not be possible without it.** CoE offers an extensive, crunched mapping for weights, items, affixes, etc. Moreover I integrated CoE's Emulator Export outputs to parse the starting/target item, offering an external, easy capture of item information over a GUI. Since I as noob needed something hands-on, easy to use, CraftOfExile was essential for this project.

- [poe.ninja](https://poe.ninja/) for providing a public API to fetch up-to-date currency exchange prices. Used by CraftPath to calculate the cost of a crafting path, and subsequently corresponding cost-based analysis. Cudos for hosting and keeping a free, public API alive for such a long time!


## Disclaimer
**CraftPath is not affiliated with or endorsed by Grinding Gear Games**

## License
[MIT License](https://github.com/WladHD/pyoe2-craftpath/blob/main/LICENSE)

[^1]: Source: trust me, bro

[^2]: Actually item state ([ItemSnapshot](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/api/item.rs)), but in the given example w/e. The item state contains more information like rarity, base item id, level, etc. 