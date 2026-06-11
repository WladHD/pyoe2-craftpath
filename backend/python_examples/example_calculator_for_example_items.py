import pyoe2_craftpath as pc
import json
from pathlib import Path

COE_MAP = {
    "./cache/coe2.json": "https://www.craftofexile.com/json/poe2/main/poec_data.json",
}

ECONOMY_MAP = {
    "./cache/pn_abyss.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Abyss",
    "./cache/pn_currency.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Currency",
    "./cache/pn_essences.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Essences",
    "./cache/pn_ritual.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Ritual"
}


def main():
    # group 1. item provider
    raw_fetched_responses_coe = pc.retrieve_contents_from_urls_with_cache_unstable_order(
        cache_url_map=COE_MAP,
        max_cache_duration_in_sec=60 * 60 * 24
    )

    # group 2. economy, order irrelevant so its fine if resulting list is in random order
    raw_fetched_responses_economy = pc.retrieve_contents_from_urls_with_cache_unstable_order(
        cache_url_map=ECONOMY_MAP,
        max_cache_duration_in_sec=60 * 60
    )

    # parse raw contents
    coe_data = pc.CraftOfExileItemInfoProvider.parse_from_json(
        raw_fetched_responses_coe[0])
    economy = pc.PoeNinjaMarketPriceProvider.parse_from_json_list(
        raw_fetched_responses_economy)

    # load raw item snapshot json created and exported from Emulator in https://www.craftofexile.com/
    with open('example_items/start_item_magic_1_affix_bow.json', 'r', encoding='utf-8') as f:
        start_raw_string = f.read()
    with open('example_items/target_item_desecrated_essence_rare_4_affix_bow.json', 'r', encoding='utf-8') as f:
        end_raw_string = f.read()

    # parse item snapshot
    start_item = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
        start_raw_string, coe_data)
    end_item = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
        end_raw_string, coe_data)

    # prettyprint out item
    print(start_item.to_pretty_string(coe_data, True))
    print(end_item.to_pretty_string(coe_data, True))

    # select instance responsible for creating the item matrix
    # currently only HappyPathMatrixBuilder available
    # (or ofc yours, if you create a Rust addon which creates a `DynMatrixBuilder`)
    group_chance_instance = pc.MatrixBuilderPreset.HappyPathMatrixBuilder.get_instance()
    print(group_chance_instance)

    # calculate item matrix, now contained in calc
    calc = pc.Calculator.generate_item_matrix(
        starting_item=start_item,
        target=end_item,
        item_provider=coe_data,
        market_info=economy,
        matrix_builder=pc.MatrixBuilderPreset.HappyPathMatrixBuilder.get_instance())

    print("Matrix contains", calc.matrix.__len__(), "items")

    # collect best 10 routes by chance
    unique_path_chance_instance = pc.StatisticAnalyzerPathPreset.UniquePathChance.get_instance()
    print(unique_path_chance_instance)
    unique_path_chance_res = calc.calculate_statistics(
        item_provider=coe_data,
        market_provider=economy,
        max_routes=5,
        max_ram_in_bytes=1000000000,  # 1 GB
        statistic_analyzer=unique_path_chance_instance)

    # collect best 10 routes by efficiency
    # (cost per single craft * tries needed to get the desired item for at least 60%)
    unique_path_efficiency_instance = pc.StatisticAnalyzerPathPreset.UniquePathEfficiency.get_instance()
    print(unique_path_efficiency_instance)
    unique_path_efficiency_res = calc.calculate_statistics(
        item_provider=coe_data,
        market_provider=economy,
        max_routes=5,
        max_ram_in_bytes=1000000000,  # 1 GB
        statistic_analyzer=unique_path_efficiency_instance)

    # collect best 10 routes by cost per single craft
    unique_path_cost_instance = pc.StatisticAnalyzerPathPreset.UniquePathCost.get_instance()
    print(unique_path_cost_instance)
    unique_path_cost_res = calc.calculate_statistics(
        item_provider=coe_data,
        market_provider=economy,
        max_routes=5,
        max_ram_in_bytes=1000000000,  # 1 GB
        statistic_analyzer=unique_path_cost_instance)

    # Collect all possible currency-sequences, sorted by chance
    # This is neat, cauz it shows you the actual chance of a given sequence.
    # Unique paths will tell you, which chance that *EXACT* path will have.
    # The group chance combines all those paths into a currency-sequence and will tell you the exact chance of the group.
    # *Theoretical example* if there exists two paths "Exalted Orb -> Exalted Orb" that result in item X for 50 % chance,
    # the currency-sequence "Exalted Orb -> Exalted Orb" will have a 100 % chance. Hence, you can apply two Exalted Orbs,
    # to reach the desired item.
    #
    # TRADEOFF: this method can very easily surpass 4 GB of RAM for 5 affixes+ *without desecration and essences*.
    # For 6 its almost guaranteed.
    # Esp. for this method `max_ram_in_bytes` is needed, to make sure it uses maximally as much memory as you're comfortable with.
    group_chance_instance = pc.StatisticAnalyzerCurrencyGroupPreset.CurrencyGroupChance.get_instance()
    groups = calc.calculate_statistics_currency_group(
        item_provider=coe_data,
        market_provider=economy,
        max_ram_in_bytes=1000000000,  # 1 GB
        statistic_analyzer=group_chance_instance
    )

    # Pretty print group info
    for index, g in enumerate(groups[:3]):
        print("Group #", index + 1, "-", g.to_pretty_string(
            item_provider=coe_data,
            market_provider=economy,
            statistic_analyzer=group_chance_instance
        ))

    # Print out calculated results ... or convert to your own desired data structure
    for (results, analyzer_instance) in [
            (unique_path_chance_res, unique_path_chance_instance),
            (unique_path_efficiency_res, unique_path_efficiency_instance),
            (unique_path_cost_res, unique_path_chance_instance)
    ]:
        for route in results[:3]:
            # if you want to look into the corresponding currency-sequence group yourself
            group = route.locate_group(calculated_groups=groups)
            if group is not None:
                print("Manual lookup group chance: ", group.chance)

            pretty = route.to_pretty_string(
                item_provider=coe_data,
                market_provider=economy,
                calculator=calc,
                # If you didn't calculate groups (for RAM reasons, or just not needed, replace with None)
                groups=groups,
                statistic_analyzer=analyzer_instance,
            )
            print(pretty)


if __name__ == "__main__":
    main()
