"""The classic example, rewritten with the new engine layer.

Same scenario as example_calculator_for_example_items.py — fetch league data,
parse the example bow items, find the best routes — in a few lines.
"""

import pyoe2_craftpath as pc

POE2_LEAGUE = "Standard"

engine = pc.LocalEngine(cache_dir="./cache")

# parsing CoE-emulator exports still needs the item info provider
item_provider, _ = engine.providers(POE2_LEAGUE)
start = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
    open("example_items/start_item_magic_1_affix_bow.json").read(), item_provider
)
target = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
    open("example_items/expensive_target_item_rare_6_affix_bow.json").read(), item_provider
)

result = engine.run(
    pc.JobSpec(
        start=start,
        target=target,
        league=POE2_LEAGUE,
        path_analyzers=[
            pc.StatisticAnalyzerPathPreset.UniquePathChance,
            pc.StatisticAnalyzerPathPreset.UniquePathEfficiency,
            pc.StatisticAnalyzerPathPreset.UniquePathCost,
        ],
        group_analyzers=[pc.StatisticAnalyzerCurrencyGroupPreset.CurrencyGroupChance],
        max_routes=5,
        max_ram_in_bytes=1_000_000_000,
    )
)

print(f"matrix size: {result.matrix_size}")
print(result.pretty_text)
