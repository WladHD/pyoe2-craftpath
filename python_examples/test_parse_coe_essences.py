import pyoe2_craftpath as pc
from pyoe2_craftpath import AffixId

COE_CACHE_MAP = {
    "./cache/coe2.json": "https://www.craftofexile.com/json/poe2/main/poec_data.json"
}

CACHE_TTL_IN_SECONDS = 60 * 60 * 24  # 1 day in seconds, coe doesnt change often


def main():
    text = pc.retrieve_contents_from_urls_with_cache_unstable_order(
        COE_CACHE_MAP, CACHE_TTL_IN_SECONDS)[0]
    data = pc.CraftOfExileItemInfoProvider.parse_from_json(text)

    bow_base_item_id = pc.BaseItemId(20)

    #####################
    # TESTING ESSENCE ONLY AFFIX
    #####################
    attack_speed_for_5697 = AffixId(5697)
    essence_id_for_5697 = data.lookup_affix_essences(
        attack_speed_for_5697, bow_base_item_id)
    print(essence_id_for_5697)

    base_mods_for_5697_20 = data.lookup_base_item_mods(pc.BaseItemId(20))
    possible_defs = base_mods_for_5697_20.get(attack_speed_for_5697)

    assert possible_defs is not None

    assert essence_id_for_5697.__contains__(
        # Greater Essence of Haste
        pc.EssenceId(3156))

    for essence in essence_id_for_5697:
        definition = data.lookup_essence_definition(essence)

        print("Printing definition for essence: ", definition.name_essence)

        possible_defs = definition.base_tier_table.get(pc.BaseItemId(20))

        assert possible_defs is not None

        for i, (k, v) in enumerate(possible_defs.items()):
            print("#", i + 1, " ", k, " : ", v)

            defi = data.lookup_base_item_mods(bow_base_item_id)

            info = defi.get(attack_speed_for_5697)

            assert info is not None

            equivalent_affix_tier = next(((k2, v2) for k2, v2 in info.items(
            ) if v2.min_item_level == v.min_item_level), None)

            assert equivalent_affix_tier is not None

            print("Will achieve tier: ", equivalent_affix_tier[0])

    #####################
    # TESTING NORMAL AFFIX; THAT CAN BE REACHED WITH AN ESSENCE
    #####################
    attack_speed = AffixId(5092)
    essence_id_for_5092 = data.lookup_affix_essences(
        attack_speed, bow_base_item_id)

    # should exactly be 2 (lesser + normal essence of haste)
    assert essence_id_for_5092.__len__() == 2

    # thats currently how CoEs structure is mapped. following works:
    assert essence_id_for_5092.__contains__(
        pc.EssenceId(3132))  # Lesser Essence of Haste

    assert essence_id_for_5092.__contains__(
        pc.EssenceId(3144))  # Essence of Haste
    assert not essence_id_for_5092.__contains__(
        # Greater Essence of Haste <- this should belong to 5697 not 5092
        pc.EssenceId(3156))

    for essence in essence_id_for_5092:
        definition = data.lookup_essence_definition(essence)

        print("Printing definition for essence: ", definition.name_essence)

        possible_defs = definition.base_tier_table.get(pc.BaseItemId(20))

        assert possible_defs is not None

        for i, (k, v) in enumerate(possible_defs.items()):
            print("#", i + 1, " ", k, " : ", v)

            defi = data.lookup_base_item_mods(bow_base_item_id)

            info = defi.get(attack_speed)

            assert info is not None

            equivalent_affix_tier = next(((k2, v2) for k2, v2 in info.items(
            ) if v2.min_item_level == v.min_item_level), None)

            assert equivalent_affix_tier is not None

            print("Will achieve tier: ", equivalent_affix_tier[0])

    # Greater Essence of Haste -> has own essence-only affix
    essence = data.lookup_essence_definition(pc.EssenceId(3156))

    assert essence is not None

    print(essence)

    # If this test finishes without errors, it works as intended


if __name__ == "__main__":
    main()
