import pyoe2_craftpath as pc
from pyoe2_craftpath import AffixId
from pprint import pprint

COE_CACHE_MAP = {
    "./cache/coe2.json": "https://www.craftofexile.com/json/poe2/main/poec_data.json"
}

CACHE_TTL_IN_SECONDS = 60 * 60 * 24  # 1 day in seconds, coe doesnt change often


def main():
    text = pc.retrieve_contents_from_urls_with_cache_unstable_order(
        COE_CACHE_MAP, CACHE_TTL_IN_SECONDS)[0]

    # redundand, since its handled automatically in
    # parse_item_data_from_json too, but just fyi
    if text.startswith("poecd="):
        text = text[len("poecd="):]

    ########################################
    ###     this is the magic line         #
    ########################################
    data = pc.CraftOfExileItemInfoProvider.parse_from_json(text)

    # Everything else just checks validity
    assert (AffixId(5) == AffixId(5))

    # % increased Lightning Damage, Prefix, Base
    affix_def = data.cache_affix_def.get(AffixId(5209))
    assert (affix_def != None)

    pprint(AffixId(5))  # out: <builtins.AffixId object at 0xADDR>
    print(AffixId(5))  # out: AffixId(5)

    print(affix_def)  # should print out all data nicely
    # e. g. v0.1.0 (AffixDefinition { exlusive_groups: {"LightningDamagePercentage"},
    # tags: {33, 20, 8}, description_template: "#% increased Lightning Damage", affix_class: Base, affix_location: Prefix })

    assert (affix_def.affix_class == pc.AffixClassEnum.Base)
    assert (affix_def.affix_location == pc.AffixLocationEnum.Prefix)

    assert (affix_def.exlusive_groups.intersection(
        ["LightningDamagePercentage"]))

    assert (not affix_def.exlusive_groups.intersection(
        ["watisdis"]))


if __name__ == "__main__":
    main()
