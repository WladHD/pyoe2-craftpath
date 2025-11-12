# *Py*oE 2 - CraftPath
A tool for Path of Exile 2 to find the best craftpaths based on the categories: *most likely, most efficient and cheapest*, between a starting item and a target item.

Available as the Python package [`pyoe2-craftpath`](https://pypi.org/project/pyoe2-craftpath/) or as "bro just gimme something that goes brrr"-executable command-line utility for Windows under [Releases](https://github.com/WladHD/pyoe2-craftpath/releases). Bindings for Python are generated with [PyO3](https://github.com/PyO3/pyo3), to let you build your own data analysis pipeline upon the calculated items and craftpaths. Made possible by the power of [*🦀 Rust*](https://www.reddit.com/r/linuxmemes/comments/1b7y5vv/rust/).

## About (Me)
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



## Strategy

## Disclaimer
**CraftPath is not affiliated with or endorsed by Grinding Gear Games**

## License
[MIT License](https://github.com/WladHD/pyoe2-craftpath/blob/main/LICENSE)

[^1]: Source: trust me, bro