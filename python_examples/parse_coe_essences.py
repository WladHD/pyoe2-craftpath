import pyoe2_craftpath as pc
from pyoe2_craftpath import AffixId
from pprint import pprint
import os
import requests

COE_CACHE_MAP = {
    "./cache/coe2.json": "https://www.craftofexile.com/json/poe2/main/poec_data.json"
}

CACHE_TTL_IN_SECONDS = 60 * 60 * 24  # 1 day in seconds, coe doesnt change often


def main():
    text = pc.retrieve_jsons_from_urls_with_cache(
        COE_CACHE_MAP, CACHE_TTL_IN_SECONDS)[0]
    data = pc.parse_item_data_from_json(text)

    attack_speed_for_5697 = AffixId(5697)
    essence_id_for_5697 = data.lookup_affix_essence(attack_speed_for_5697)
    print(essence_id_for_5697)

    base_mods_for_5697_20 = data.lookup_base_item_mods(pc.BaseItemId(20))
    possible_defs = base_mods_for_5697_20.get(attack_speed_for_5697)

    assert possible_defs is not None

    for k, v in possible_defs.items():
        print(k, " : ", v)

    assert essence_id_for_5697.__contains__(
        # Greater Essence of Haste
        pc.EssenceId(3156))

    attack_speed = AffixId(5092)
    essence_id_for_5092 = data.lookup_affix_essence(
        attack_speed)

    # should NOT be 1 .. but currently is not implemented fully
    assert essence_id_for_5092.__len__() > 1

    # thats currently how CoEs structure is mapped. following works:
    assert essence_id_for_5092.__contains__(
        pc.EssenceId(3132))  # Lesser Essence of Haste

    # THIS DOES NOT WORK YET:
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

        for k, v in possible_defs.items():
            print(k, " : ", v)

    # CURRENTLY FAILS; SINCE ITS NOT LINKED WITH THE TOP ITEM YET. Only lesser hastes are linked right now,
    # needs to be resolved later.
    essence = data.lookup_essence_definition(pc.EssenceId(3144))

    # If this test resolves without errors, it has been resolved
    print(essence)


if __name__ == "__main__":
    main()
