import pyoe2_craftpath as pc
from pyoe2_craftpath import AffixId
from pprint import pprint
import os
import requests
import time

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
    raw_fetched_responses_coe = pc.retrieve_contents_from_urls_with_cache_unstable_order(
        cache_url_map=COE_MAP,
        max_cache_duration_in_sec=60 * 60 * 24
    )

    raw_fetched_responses_economy = pc.retrieve_contents_from_urls_with_cache_unstable_order(
        cache_url_map=ECONOMY_MAP,
        max_cache_duration_in_sec=60 * 60
    )

    coe_data = pc.CraftOfExileItemInfoProvider.parse_from_json(
        raw_fetched_responses_coe[0])
    economy = pc.PoeNinjaMarketPriceProvider.parse_from_json_list(
        raw_fetched_responses_economy)

    # clean approach would be to save this to a file and load it.
    # to ease reproducibility item jsons are hardtyped here.
    start_item = '{"settings":{"bgroup":7,"base":20,"bitem":232,"ilvl":100,"rarity":"normal","influences":null,"sockets":0,"socketed":[],"quality":20,"exmods":null},"params":{"mode":null,"currency":null,"action":null,"subaction":null,"ssaction":null,"disabled":"|catalyst|poe2_desecrationancient_ribs|poe2_desecrationgnawed_ribs|poe2_desecrationpreserved_ribs|poe2_desecrationancient_collarbone|poe2_desecrationgnawed_collarbone|poe2_desecrationpreserved_collarbone|poe2_desecrationpreserved_cranium|poe2_desecrationpreserved_spine|poe2_desecrationgnawed_collarbone|poe2_desecrationgnawed_ribs|poe2_desecrationgnawed_jawbone|fossil18|eldritch|vendor|","cursor":""},"data":{"implicits":null,"iaffixes":[],"iaffbt":{"prefix":0,"suffix":0},"imprint":null,"eldritch":null,"meta_flags":{}},"log":[],"spending":{"currency":{"transmute":0,"augmentation":0,"regal":0,"poe2_alchemy":0,"chaos":0,"exalted":0,"annul":0,"divine":0,"artificer":0,"fracturing":0,"vaal":0},"actions":{"transmute":{"normal":0,"greater":0,"perfect":0},"augmentation":{"normal":0,"greater":0,"perfect":0},"regal":{"normal":0,"greater":0,"perfect":0},"chaos":{"normal":0,"greater":0,"perfect":0},"exalted":{"normal":0,"greater":0,"perfect":0},"poe2_lesser_essence":{"3122":0,"3123":0,"3124":0,"3125":0,"3126":0,"3127":0,"3128":0,"3129":0,"3130":0,"3131":0,"3132":0,"3133":0,"3174":0,"3178":0,"3182":0,"3186":0,"3190":0,"3194":0,"3198":0},"poe2_essence":{"3134":0,"3135":0,"3136":0,"3137":0,"3138":0,"3139":0,"3140":0,"3141":0,"3142":0,"3143":0,"3144":0,"3145":0,"3175":0,"3179":0,"3183":0,"3187":0,"3191":0,"3195":0,"3199":0},"poe2_greater_essence":{"3146":0,"3147":0,"3148":0,"3149":0,"3150":0,"3151":0,"3152":0,"3153":0,"3154":0,"3155":0,"3156":0,"3157":0,"3176":0,"3180":0,"3184":0,"3188":0,"3192":0,"3196":0,"3200":0},"poe2_perfect_essence":{"3158":0,"3159":0,"3160":0,"3161":0,"3162":0,"3163":0,"3164":0,"3165":0,"3166":0,"3167":0,"3168":0,"3169":0,"3170":0,"3171":0,"3172":0,"3173":0,"3177":0,"3181":0,"3185":0,"3189":0,"3193":0,"3197":0,"3201":0},"poe2_desecration":{"ancient_collarbone":0,"ancient_jawbone":0,"ancient_ribs":0,"gnawed_collarbone":0,"gnawed_jawbone":0,"gnawed_ribs":0,"preserved_collarbone":0,"preserved_cranium":0,"preserved_jawbone":0,"preserved_ribs":0,"preserved_spine":0},"poe2_omens":{"abyssal_echoes":0,"blackblooded":0,"blessed":0,"dextral_annulment":0,"dextral_crystallisation":0,"dextral_erasure":0,"dextral_exaltation":0,"dextral_necromancy":0,"greater_exaltation":0,"homogenising_exaltation":0,"homogenising_coronation":0,"liege":0,"light":0,"sinistral_annulment":0,"sinistral_crystallisation":0,"sinistral_erasure":0,"sinistral_exaltation":0,"sinistral_necromancy":0,"sovereign":0,"whittling":0},"poe2_runes":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0,"13":0,"14":0,"15":0,"16":0,"17":0,"18":0,"19":0,"20":0,"21":0,"22":0,"23":0,"24":0,"25":0,"26":0,"27":0,"28":0,"29":0,"30":0,"46":0,"47":0,"48":0,"49":0,"50":0,"51":0,"52":0,"53":0,"54":0,"55":0,"56":0,"57":0,"58":0,"59":0,"60":0,"61":0,"73":0,"74":0,"75":0,"76":0,"77":0,"78":0,"79":0,"80":0,"81":0,"82":0,"83":0,"84":0,"85":0,"86":0,"87":0,"88":0,"89":0,"90":0,"91":0,"92":0,"93":0,"lesser":0,"normal":0,"greater":0,"special":0},"poe2_cores":{"31":0,"32":0,"33":0,"34":0,"35":0,"36":0,"37":0,"38":0,"39":0,"40":0,"41":0,"42":0,"43":0,"44":0,"45":0,"101":0,"102":0,"103":0,"104":0,"105":0,"106":0,"107":0,"108":0,"109":0,"110":0,"111":0,"112":0,"113":0,"114":0,"115":0,"116":0,"117":0,"118":0,"119":0},"poe2_talismans":{"62":0,"63":0,"64":0,"65":0,"66":0,"67":0,"68":0,"69":0,"70":0,"71":0,"72":0,"94":0,"95":0,"96":0,"97":0,"98":0,"99":0,"100":0},"catalyst":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0}}},"opened":{"mp":{"prefix":null,"suffix":null},"cb":{"prefix":null,"suffix":null},"im":{"eldritch_red":null,"eldritch_blue":null,"corrupted":null,"socket":null}},"catalyst":null}'
    end_item = '{"settings":{"bgroup":7,"base":20,"bitem":232,"ilvl":100,"rarity":"magic","influences":null,"sockets":0,"socketed":[],"quality":20,"exmods":null},"params":{"mode":null,"currency":null,"action":null,"subaction":"normal","ssaction":null,"disabled":"|catalyst|poe2_desecrationancient_ribs|poe2_desecrationgnawed_ribs|poe2_desecrationpreserved_ribs|poe2_desecrationancient_collarbone|poe2_desecrationgnawed_collarbone|poe2_desecrationpreserved_collarbone|poe2_desecrationpreserved_cranium|poe2_desecrationpreserved_spine|poe2_desecrationgnawed_collarbone|poe2_desecrationgnawed_ribs|poe2_desecrationgnawed_jawbone|fossil18|eldritch|vendor|","cursor":"url() 39 39, default"},"data":{"implicits":null,"iaffixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704},{"id":"5117","mgrp":"1","tindex":3,"atype":"prefix","modgroups":["LightningDamage"],"weight":1200,"nvalues":"[[1,2],[36,52]]","amgs":null,"tags":"|33|20|8|3|","rolls":[1,40],"bench":0,"frac":0,"maven":0,"chance":0.11663501284772651}],"iaffbt":{"prefix":1,"suffix":1},"imprint":null,"eldritch":null,"meta_flags":{}},"log":[{"mode":"currency","currency":"transmute","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":null,"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"normal","affixes":[],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":0,"suffix":0},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"augmentation","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":[{"id":"5117","mgrp":"1","tindex":3,"atype":"prefix","modgroups":["LightningDamage"],"weight":1200,"nvalues":"[[1,2],[36,52]]","amgs":null,"tags":"|33|20|8|3|","rolls":[1,40],"bench":0,"frac":0,"maven":0,"chance":0.11663501284772651}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"magic","affixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":0,"suffix":1},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1}],"spending":{"currency":{"transmute":1,"augmentation":1,"regal":0,"poe2_alchemy":0,"chaos":0,"exalted":0,"annul":0,"divine":0,"artificer":0,"fracturing":0,"vaal":0},"actions":{"transmute":{"normal":0,"greater":0,"perfect":0},"augmentation":{"normal":0,"greater":0,"perfect":0},"regal":{"normal":0,"greater":0,"perfect":0},"chaos":{"normal":0,"greater":0,"perfect":0},"exalted":{"normal":0,"greater":0,"perfect":0},"poe2_lesser_essence":{"3122":0,"3123":0,"3124":0,"3125":0,"3126":0,"3127":0,"3128":0,"3129":0,"3130":0,"3131":0,"3132":0,"3133":0,"3174":0,"3178":0,"3182":0,"3186":0,"3190":0,"3194":0,"3198":0},"poe2_essence":{"3134":0,"3135":0,"3136":0,"3137":0,"3138":0,"3139":0,"3140":0,"3141":0,"3142":0,"3143":0,"3144":0,"3145":0,"3175":0,"3179":0,"3183":0,"3187":0,"3191":0,"3195":0,"3199":0},"poe2_greater_essence":{"3146":0,"3147":0,"3148":0,"3149":0,"3150":0,"3151":0,"3152":0,"3153":0,"3154":0,"3155":0,"3156":0,"3157":0,"3176":0,"3180":0,"3184":0,"3188":0,"3192":0,"3196":0,"3200":0},"poe2_perfect_essence":{"3158":0,"3159":0,"3160":0,"3161":0,"3162":0,"3163":0,"3164":0,"3165":0,"3166":0,"3167":0,"3168":0,"3169":0,"3170":0,"3171":0,"3172":0,"3173":0,"3177":0,"3181":0,"3185":0,"3189":0,"3193":0,"3197":0,"3201":0},"poe2_desecration":{"ancient_collarbone":0,"ancient_jawbone":0,"ancient_ribs":0,"gnawed_collarbone":0,"gnawed_jawbone":0,"gnawed_ribs":0,"preserved_collarbone":0,"preserved_cranium":0,"preserved_jawbone":0,"preserved_ribs":0,"preserved_spine":0},"poe2_omens":{"abyssal_echoes":0,"blackblooded":0,"blessed":0,"dextral_annulment":0,"dextral_crystallisation":0,"dextral_erasure":0,"dextral_exaltation":0,"dextral_necromancy":0,"greater_exaltation":0,"homogenising_exaltation":0,"homogenising_coronation":0,"liege":0,"light":0,"sinistral_annulment":0,"sinistral_crystallisation":0,"sinistral_erasure":0,"sinistral_exaltation":0,"sinistral_necromancy":0,"sovereign":0,"whittling":0},"poe2_runes":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0,"13":0,"14":0,"15":0,"16":0,"17":0,"18":0,"19":0,"20":0,"21":0,"22":0,"23":0,"24":0,"25":0,"26":0,"27":0,"28":0,"29":0,"30":0,"46":0,"47":0,"48":0,"49":0,"50":0,"51":0,"52":0,"53":0,"54":0,"55":0,"56":0,"57":0,"58":0,"59":0,"60":0,"61":0,"73":0,"74":0,"75":0,"76":0,"77":0,"78":0,"79":0,"80":0,"81":0,"82":0,"83":0,"84":0,"85":0,"86":0,"87":0,"88":0,"89":0,"90":0,"91":0,"92":0,"93":0,"lesser":0,"normal":0,"greater":0,"special":0},"poe2_cores":{"31":0,"32":0,"33":0,"34":0,"35":0,"36":0,"37":0,"38":0,"39":0,"40":0,"41":0,"42":0,"43":0,"44":0,"45":0,"101":0,"102":0,"103":0,"104":0,"105":0,"106":0,"107":0,"108":0,"109":0,"110":0,"111":0,"112":0,"113":0,"114":0,"115":0,"116":0,"117":0,"118":0,"119":0},"poe2_talismans":{"62":0,"63":0,"64":0,"65":0,"66":0,"67":0,"68":0,"69":0,"70":0,"71":0,"72":0,"94":0,"95":0,"96":0,"97":0,"98":0,"99":0,"100":0},"catalyst":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0}}},"opened":{"mp":{"prefix":null,"suffix":null},"cb":{"prefix":null,"suffix":null},"im":{"eldritch_red":null,"eldritch_blue":null,"corrupted":null,"socket":null}},"catalyst":null}'

    start_item = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
        start_item, coe_data)
    end_item = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
        end_item, coe_data)

    print(start_item)
    print(end_item)

    group_chance_instance = pc.MatrixBuilderPreset.HappyPathMatrixBuilder.get_instance()

    print(group_chance_instance)

    calc = pc.Calculator.generate_item_matrix(
        starting_item=start_item,
        target=end_item,
        item_provider=coe_data,
        market_info=economy,
        matrix_builder=pc.MatrixBuilderPreset.HappyPathMatrixBuilder.get_instance())

    print("Matrix contains", calc.matrix.__len__(), "items")

    unique_path_chance_instance = pc.StatisticAnalyzerPathPreset.UniquePathChance.get_instance()
    group_chance_instance = pc.StatisticAnalyzerCurrencyGroupPreset.CurrencyGroupChance.get_instance()

    res = calc.calculate_statistics(
        item_provider=coe_data,
        market_provider=economy,
        max_routes=5,
        max_ram_in_bytes=1000000000,  # 1 GB
        statistic_analyzer=unique_path_chance_instance)

    groups = calc.calculate_statistics_currency_group(
        item_provider=coe_data,
        market_provider=economy,
        max_ram_in_bytes=1000000000,  # 1 GB
        statistic_analyzer=group_chance_instance
    )

    for index, g in enumerate(groups[:3]):
        print("Group #", index + 1, "-", g.to_pretty_string(
            item_provider=coe_data,
            market_provider=economy,
            statistic_analyzer=group_chance_instance
        ))

    for route in res[:2]:
        pretty = route.to_pretty_string(
            item_provider=coe_data,
            market_provider=economy,
            calculator=calc,
            groups=groups,
            statistic_analyzer=unique_path_chance_instance,
        )

        group = route.locate_group(calculated_groups=groups)

        if group is not None:
            print("Manual lookup group: ", group.chance)

        print(pretty)

    for route in res[2:]:
        pretty = route.to_pretty_string(
            item_provider=coe_data,
            market_provider=economy,
            calculator=calc,
            groups=None,
            statistic_analyzer=pc.StatisticAnalyzerPathPreset.UniquePathChance.get_instance(),
        )

        print(pretty)


if __name__ == "__main__":
    main()
