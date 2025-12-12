import pyoe2_craftpath as pc

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
    with open('example_items/startitem_illegal_desecrator.json', 'r', encoding='utf-8') as f:
        start_raw_string = f.read()
    with open('example_items/targetitem_illegal_desecrator.json', 'r', encoding='utf-8') as f:
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

    try:
        # calculate item matrix, now contained in calc
        calc = pc.Calculator.generate_item_matrix(
            starting_item=start_item,
            target=end_item,
            item_provider=coe_data,
            market_info=economy,
            matrix_builder=pc.MatrixBuilderPreset.HappyPathMatrixBuilder.get_instance())

        assert False, "Matrix propagation did not fail, but should"
    except Exception as e:
        print("Matrix propagation failed as expected: ", e)


if __name__ == "__main__":
    main()
