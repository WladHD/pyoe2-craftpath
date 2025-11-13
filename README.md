# *Py*oE 2 - CraftPath
A tool for Path of Exile 2 to find the best craftpaths based on the categories: *most likely, most efficient and cheapest*, between a starting item and a target item.

Available as Python package [`pyoe2-craftpath`](https://pypi.org/project/pyoe2-craftpath/) or as "bro just gimme something that goes brrr"-executable command-line utility for Windows under [Releases](https://github.com/WladHD/pyoe2-craftpath/releases). Bindings for Python are generated with [PyO3](https://github.com/PyO3/pyo3), to let you build your own data analysis pipeline upon the calculated items and craftpaths. Made possible by the power of [*🦀 Rust*](https://www.reddit.com/r/linuxmemes/comments/1b7y5vv/rust/).

## About
To keep it short, I was introduced to Path of Exile 2 and enjoyed it quite a bit.
After reaching higher levels and starting to get the hang of things, I became interested in crafting.
As big noob, I was completly overwhelmed with the information available.

**Me need simple. Me want good item. How get good item?**

*CraftPath*. The purpose of this tool is give it information about your current item and the affixes you want it to have, then let it efficiently calculate possible craftpaths; Without the need to manually look up mod weights, mod groups, or spend hours crunching probabilities on a Casio calculator, as all true PoE gamers do[^1].

## Features
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

## How To Run
This guide here shows the quick-n-dirty approach to run this tool via the console. This tool is actually intended to be used as a Python package. Refer to the [extended Python example](https://github.com/WladHD/pyoe2-craftpath/blob/main/python_examples/example_calculator_for_example_items.ipynb) or just skim through the `python_examples` directory.

The following shows the usage for the console on Windows.
- First things first. Download the program from the [Releases](https://github.com/WladHD/pyoe2-craftpath/releases).

Since this is a command-line utility, it should be started over the console, with: `pyoe2-craftpath.exe [available, optional arguments]`

Available, optional arguments:
- `--start_item_path <Path to JSON File>` (*if unset:* `pyoe2-craftpath/startitem.json`) - provides the file location of the saved item, to treat as the starting point of the craft. Use the export function of **[CraftOfExile](https://www.craftofexile.com/?game=poe2)'s *Emulator*** and copy-paste content to `poe2-craftpath/startitem.json`.
- `--target_item_path <Path to JSON File>` (*if unset:* `pyoe2-craftpath/targetitem.json`) - provides the file location of the saved item, to treat as the end point of the craft. Use the export function of **[CraftOfExile](https://www.craftofexile.com/?game=poe2)'s *Emulator*** and copy-paste content to `poe2-craftpath/targetitem.json`.
- `--cache_path <Path to Temp Folder>` (*if unset:* `pyoe2-craftpath`) - used for caching CraftOfExile's and PoE.Ninjas datasets. Needs to be explicitly created to avoid confusion.  
- `--poe2_league <League>` (*if unset:* `Rise of the Abyssal`) - fetches PoE.Ninjas economy data for the specified league. 
- `--amount_routes <Number>` (*if unset:* `5`) - amount of craft paths collected and printed per stats category. (Current stats categories: highest chance, most efficient, cheapest, so `3x5 = 15` shown routes with default settings).
- `--no_updates` (*if unset: checks for updates*) - if this flag is set, *CraftPath* will not query GitHub and not check for newer versions.
- `--no_groups` (*if unset: collects all possible paths*) - if this flag is set, *CraftPath* will not collect all possible paths. (Massive reduction in memory usage, but less overall gained information).
- `--max-ram <<Number> GB,KB,MB>` (*if unset: `1GB`*) - sets the maximum RAM the program is allowed to use during path collection.

**``--max-ram`` is spicy, please read**: 
The algorithm for **group collection** (disabled with `no_groups` flag) currently calculates **all** unique craft paths to create statistical info for *currency sequences* (= "*groups*"). Although *CraftPath* is relying on the `Happy Path` as main efficiency boost, it still is exponentially creating paths. **It currently lacks optimization with craft path depth higher than 5/6, easily using more then *60 GB* (!)** *e. g. (rare item with [1 unwanted affix / 5 open] into rare item with 6 wanted affixes).* To force users to activly acknowledge this (hopefully temporary) constraint, I set the RAM to `1GB`. I feel like this is a natural barrier that most users will be fine with, while still allowing the usual propagation of 1 - 4 steps. And even 5 - 6 steps if desecration is involved. Bruteforce-calculating long paths will trigger the program to abort - forcing you to eventually read this, at least :D 

*For devs*: the high RAM usage occurs in the collection of *all possible paths* in [calculate_currency_groups](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/statistics/statistic_analyzer_currency_grouped_collector.rs#L22) as references, but allocating new vectors for every path; This is a design decision to firstly filter out circularity and secondly allow grouping of paths by *currency sequence*, providing broader insight into which *currency sequences* are generally the best overall - rather than focusing only on a *single* most likely crafting path, which might not belong to the most likely *currency sequence*. If you have ideas on how to constrain or filter deep craft paths while still keeping the grouped comparison aspect, feel free to open an issue ... or provide an own implementation of [`StatisticAnalyzerCurrencyGroups`](https://github.com/WladHD/pyoe2-craftpath/blob/main/src/calc/statistics/analyzers/currency_group_chance_statistic_analyzer.rs) :)

## Development Strategy and Caveats<a name="strategy-and-development"></a>
To constraint propagation and massivly reduce complexity, my algorithm tries to stay on the `Happy Path` as much as possible. That means, that affixes that can be rolled, but are not included in the desired *affix state*, will not be considered for additive currencies like `Exalted Orb`. Subtractive currencies like `Orb of Annulment` will only result in *affix states*, that lose unwanted affixes. **Simply put, if this algorithm was a player, it would immediatly stop crafting an item, *that does not gain an affix from the desired affixes (or lose an unwanted affix)***.

While this approach enables more efficient path construction, it may miss routes that can only be reached by temporarily applying an undesired affix. Such an edge case can be found by trying to apply `Perfect Essence`. 

Let's assume we have an item with three desirable prefixes that we want to keep, and we plan to apply a `Perfect Essence` to add a suffix. Naivly applying it results in an item with two prefixes and the new suffix from the `Perfect Essence`. This action is disallowed by *CraftPath*, since it would remove a wanted affix, thus stopping propagation and completing without finding a craft path at all.

To fix this specific edge case, *CraftPath* introduces an additional temporary step, forcing propagation outside of the `Happy Path`: it first applies a suffix from the unwanted affix pool. This ensures that the three desired prefixes remain untouched, while the temporary suffix can be replaced with the `Perfect Essence` and the `Dextral Crystallisation` omen.

I'm sure many more such edge cases exist, and those need to be specifically implemented. If you can think of any, please tell me :3

## Acknowledgments
Of course, [Grinding Gear Games](https://www.grindinggear.com/) for provinding Path of Exile 2, that got me hooked to the extent of actually coding this.

[CraftOfExile](https://www.craftofexile.com/) that permitted me to use its [item data](https://www.craftofexile.com/json/poe2/main/poec_data.json). This project would not be possible without an easily accessible weight, item, affix, etc. mapping. Moreover I integrated its Emulator outputs to parse the starting/target items.

[poe.ninja](https://poe.ninja/) for providing an open API to fetch up-to-date currency exchange prices. Used for calculating the cost of a crafting path. 

## Disclaimer
**CraftPath is not affiliated with or endorsed by Grinding Gear Games**

## License
[MIT License](https://github.com/WladHD/pyoe2-craftpath/blob/main/LICENSE)

[^1]: Source: trust me, bro