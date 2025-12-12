pub fn init_test_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt().with_target(false).try_init();
    });
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use pyoe2_craftpath::{
        api::{calculator::Calculator, types::THashMap},
        calc::{
            matrix::presets::matrix_builder_presets::MatrixBuilderPreset,
            statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset,
        },
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
            pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider,
        },
    };

    use crate::init_test_tracing;

    #[test]
    fn test_calculator_exalted_orb() -> Result<()> {
        init_test_tracing();

        let item_provider_hm = THashMap::from_iter(vec![(
            "./cache/coe2.json".to_string(),
            "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
        )]);

        let economy_provider_hm = THashMap::from_iter(vec![
            (
                "./cache/pn_abyss.json".to_string(),
                "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Abyss".to_string(),
            ),
            (
                "./cache/pn_currency.json".to_string(),
                "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Currency".to_string(),
            ),
            (
                "./cache/pn_essences.json".to_string(),
                "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Essences".to_string(),
            ),
            (
                "./cache/pn_ritual.json".to_string(),
                "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Ritual".to_string(),
            ),
        ]);

        let item_cached_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
            item_provider_hm,
            60_u64 * 60_u64 * 24_u64,
        )?;
        let economy_cached_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
            economy_provider_hm,
            60_u64 * 60_u64,
        )?;

        let item_provider =
            CraftOfExileItemInfoProvider::parse_from_json(item_cached_jsons.first().unwrap())?;
        let market_info =
            PoeNinjaMarketPriceProvider::parse_from_json_list(economy_cached_jsons.as_ref())?;

        let starting_item = r#"{"settings":{"bgroup":7,"base":20,"bitem":232,"ilvl":100,"rarity":"normal","influences":null,"sockets":0,"socketed":[],"quality":20,"exmods":null},"params":{"mode":null,"currency":null,"action":null,"subaction":null,"ssaction":null,"disabled":"|catalyst|poe2_desecrationancient_ribs|poe2_desecrationgnawed_ribs|poe2_desecrationpreserved_ribs|poe2_desecrationancient_collarbone|poe2_desecrationgnawed_collarbone|poe2_desecrationpreserved_collarbone|poe2_desecrationpreserved_cranium|poe2_desecrationpreserved_spine|poe2_desecrationgnawed_collarbone|poe2_desecrationgnawed_ribs|poe2_desecrationgnawed_jawbone|fossil18|eldritch|vendor|","cursor":""},"data":{"implicits":null,"iaffixes":[],"iaffbt":{"prefix":0,"suffix":0},"imprint":null,"eldritch":null,"meta_flags":{}},"log":[],"spending":{"currency":{"transmute":0,"augmentation":0,"regal":0,"poe2_alchemy":0,"chaos":0,"exalted":0,"annul":0,"divine":0,"artificer":0,"fracturing":0,"vaal":0},"actions":{"transmute":{"normal":0,"greater":0,"perfect":0},"augmentation":{"normal":0,"greater":0,"perfect":0},"regal":{"normal":0,"greater":0,"perfect":0},"chaos":{"normal":0,"greater":0,"perfect":0},"exalted":{"normal":0,"greater":0,"perfect":0},"poe2_lesser_essence":{"3122":0,"3123":0,"3124":0,"3125":0,"3126":0,"3127":0,"3128":0,"3129":0,"3130":0,"3131":0,"3132":0,"3133":0,"3174":0,"3178":0,"3182":0,"3186":0,"3190":0,"3194":0,"3198":0},"poe2_essence":{"3134":0,"3135":0,"3136":0,"3137":0,"3138":0,"3139":0,"3140":0,"3141":0,"3142":0,"3143":0,"3144":0,"3145":0,"3175":0,"3179":0,"3183":0,"3187":0,"3191":0,"3195":0,"3199":0},"poe2_greater_essence":{"3146":0,"3147":0,"3148":0,"3149":0,"3150":0,"3151":0,"3152":0,"3153":0,"3154":0,"3155":0,"3156":0,"3157":0,"3176":0,"3180":0,"3184":0,"3188":0,"3192":0,"3196":0,"3200":0},"poe2_perfect_essence":{"3158":0,"3159":0,"3160":0,"3161":0,"3162":0,"3163":0,"3164":0,"3165":0,"3166":0,"3167":0,"3168":0,"3169":0,"3170":0,"3171":0,"3172":0,"3173":0,"3177":0,"3181":0,"3185":0,"3189":0,"3193":0,"3197":0,"3201":0},"poe2_desecration":{"ancient_collarbone":0,"ancient_jawbone":0,"ancient_ribs":0,"gnawed_collarbone":0,"gnawed_jawbone":0,"gnawed_ribs":0,"preserved_collarbone":0,"preserved_cranium":0,"preserved_jawbone":0,"preserved_ribs":0,"preserved_spine":0},"poe2_omens":{"abyssal_echoes":0,"blackblooded":0,"blessed":0,"dextral_annulment":0,"dextral_crystallisation":0,"dextral_erasure":0,"dextral_exaltation":0,"dextral_necromancy":0,"greater_exaltation":0,"homogenising_exaltation":0,"homogenising_coronation":0,"liege":0,"light":0,"sinistral_annulment":0,"sinistral_crystallisation":0,"sinistral_erasure":0,"sinistral_exaltation":0,"sinistral_necromancy":0,"sovereign":0,"whittling":0},"poe2_runes":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0,"13":0,"14":0,"15":0,"16":0,"17":0,"18":0,"19":0,"20":0,"21":0,"22":0,"23":0,"24":0,"25":0,"26":0,"27":0,"28":0,"29":0,"30":0,"46":0,"47":0,"48":0,"49":0,"50":0,"51":0,"52":0,"53":0,"54":0,"55":0,"56":0,"57":0,"58":0,"59":0,"60":0,"61":0,"73":0,"74":0,"75":0,"76":0,"77":0,"78":0,"79":0,"80":0,"81":0,"82":0,"83":0,"84":0,"85":0,"86":0,"87":0,"88":0,"89":0,"90":0,"91":0,"92":0,"93":0,"lesser":0,"normal":0,"greater":0,"special":0},"poe2_cores":{"31":0,"32":0,"33":0,"34":0,"35":0,"36":0,"37":0,"38":0,"39":0,"40":0,"41":0,"42":0,"43":0,"44":0,"45":0,"101":0,"102":0,"103":0,"104":0,"105":0,"106":0,"107":0,"108":0,"109":0,"110":0,"111":0,"112":0,"113":0,"114":0,"115":0,"116":0,"117":0,"118":0,"119":0},"poe2_talismans":{"62":0,"63":0,"64":0,"65":0,"66":0,"67":0,"68":0,"69":0,"70":0,"71":0,"72":0,"94":0,"95":0,"96":0,"97":0,"98":0,"99":0,"100":0},"catalyst":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0}}},"opened":{"mp":{"prefix":null,"suffix":null},"cb":{"prefix":null,"suffix":null},"im":{"eldritch_red":null,"eldritch_blue":null,"corrupted":null,"socket":null}},"catalyst":null}"#;
        let target_item = r#"{"settings":{"bgroup":7,"base":20,"bitem":232,"ilvl":100,"rarity":"rare","influences":null,"sockets":0,"socketed":[],"quality":20,"exmods":null},"params":{"mode":null,"currency":null,"action":null,"subaction":"normal","ssaction":null,"disabled":"|catalyst|poe2_desecrationancient_ribs|poe2_desecrationgnawed_ribs|poe2_desecrationpreserved_ribs|poe2_desecrationancient_collarbone|poe2_desecrationgnawed_collarbone|poe2_desecrationpreserved_collarbone|poe2_desecrationpreserved_cranium|poe2_desecrationpreserved_spine|poe2_desecrationgnawed_collarbone|poe2_desecrationgnawed_ribs|poe2_desecrationgnawed_jawbone|fossil18|eldritch|vendor|","cursor":"url(39 39, default"},"data":{"implicits":null,"iaffixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704},{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":2,"bench":0,"frac":0,"maven":0,"nvalues":"[[25,34],[47,72]]","rolls":[29,61],"weight":"1000"},{"id":"5054","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["ManaGainedFromEnemyDeath"],"weight":750,"nvalues":"[[28,35]]","amgs":null,"tags":"|2|","rolls":[32],"bench":0,"frac":0,"maven":0,"chance":0.016469038208168644},{"id":"5115","mgrp":"1","tindex":1,"atype":"prefix","modgroups":["FireDamage"],"weight":800,"nvalues":"[[4,6],[7,10]]","amgs":null,"tags":"|33|20|6|3|","rolls":[6,7],"bench":0,"frac":0,"maven":0,"chance":0.059708509637987774},{"id":"5100","mgrp":"1","tindex":3,"atype":"suffix","modgroups":["LocalAttributeRequirements"],"weight":1000,"nvalues":"[30]","amgs":null,"tags":null,"rolls":[30],"bench":0,"frac":0,"maven":0,"chance":0.025252525252525252},{"id":"5056","mgrp":"1","tindex":0,"atype":"prefix","modgroups":["IncreasedAccuracy"],"weight":800,"nvalues":"[[11,32]]","amgs":null,"tags":"|3|","rolls":[14],"bench":0,"frac":0,"maven":0,"chance":0.1727475800446761}],"iaffbt":{"prefix":3,"suffix":3},"imprint":null,"eldritch":null,"meta_flags":{}},"log":[{"mode":"currency","currency":"transmute","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":null,"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"normal","affixes":[],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":0,"suffix":0},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"forced","currency":null,"action":"mp","ssaction":2,"subaction":5121,"forced":"add","add":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":2,"bench":0,"frac":0,"maven":0,"nvalues":"[[25,34],[47,72]]","rolls":[29,61],"weight":"1000"}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"magic","affixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":0,"suffix":1},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"regal","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":[{"id":"5054","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["ManaGainedFromEnemyDeath"],"weight":750,"nvalues":"[[28,35]]","amgs":null,"tags":"|2|","rolls":[32],"bench":0,"frac":0,"maven":0,"chance":0.016469038208168644}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"magic","affixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704},{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":2,"bench":0,"frac":0,"maven":0,"nvalues":"[[25,34],[47,72]]","rolls":[29,61],"weight":"1000"}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":1,"suffix":1},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"exalted","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":[{"id":"5115","mgrp":"1","tindex":1,"atype":"prefix","modgroups":["FireDamage"],"weight":800,"nvalues":"[[4,6],[7,10]]","amgs":null,"tags":"|33|20|6|3|","rolls":[6,7],"bench":0,"frac":0,"maven":0,"chance":0.059708509637987774}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"rare","affixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704},{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":2,"bench":0,"frac":0,"maven":0,"nvalues":"[[25,34],[47,72]]","rolls":[29,61],"weight":"1000"},{"id":"5054","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["ManaGainedFromEnemyDeath"],"weight":750,"nvalues":"[[28,35]]","amgs":null,"tags":"|2|","rolls":[32],"bench":0,"frac":0,"maven":0,"chance":0.016469038208168644}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":1,"suffix":2},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"exalted","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":[{"id":"5100","mgrp":"1","tindex":3,"atype":"suffix","modgroups":["LocalAttributeRequirements"],"weight":1000,"nvalues":"[30]","amgs":null,"tags":null,"rolls":[30],"bench":0,"frac":0,"maven":0,"chance":0.025252525252525252}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"rare","affixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704},{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":2,"bench":0,"frac":0,"maven":0,"nvalues":"[[25,34],[47,72]]","rolls":[29,61],"weight":"1000"},{"id":"5054","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["ManaGainedFromEnemyDeath"],"weight":750,"nvalues":"[[28,35]]","amgs":null,"tags":"|2|","rolls":[32],"bench":0,"frac":0,"maven":0,"chance":0.016469038208168644},{"id":"5115","mgrp":"1","tindex":1,"atype":"prefix","modgroups":["FireDamage"],"weight":800,"nvalues":"[[4,6],[7,10]]","amgs":null,"tags":"|33|20|6|3|","rolls":[6,7],"bench":0,"frac":0,"maven":0,"chance":0.059708509637987774}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":2,"suffix":2},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"exalted","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":[{"id":"5056","mgrp":"1","tindex":0,"atype":"prefix","modgroups":["IncreasedAccuracy"],"weight":800,"nvalues":"[[11,32]]","amgs":null,"tags":"|3|","rolls":[14],"bench":0,"frac":0,"maven":0,"chance":0.1727475800446761}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":232,"base":20,"bgroup":7,"rarity":"rare","affixes":[{"id":"5127","mgrp":"1","tindex":1,"atype":"suffix","modgroups":["LightRadiusAndAccuracy"],"weight":1000,"nvalues":"[10,[21,40]]","amgs":null,"tags":"|3|","rolls":[10,38],"bench":0,"frac":0,"maven":0,"chance":0.02012477359629704},{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":2,"bench":0,"frac":0,"maven":0,"nvalues":"[[25,34],[47,72]]","rolls":[29,61],"weight":"1000"},{"id":"5054","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["ManaGainedFromEnemyDeath"],"weight":750,"nvalues":"[[28,35]]","amgs":null,"tags":"|2|","rolls":[32],"bench":0,"frac":0,"maven":0,"chance":0.016469038208168644},{"id":"5115","mgrp":"1","tindex":1,"atype":"prefix","modgroups":["FireDamage"],"weight":800,"nvalues":"[[4,6],[7,10]]","amgs":null,"tags":"|33|20|6|3|","rolls":[6,7],"bench":0,"frac":0,"maven":0,"chance":0.059708509637987774},{"id":"5100","mgrp":"1","tindex":3,"atype":"suffix","modgroups":["LocalAttributeRequirements"],"weight":1000,"nvalues":"[30]","amgs":null,"tags":null,"rolls":[30],"bench":0,"frac":0,"maven":0,"chance":0.025252525252525252}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":2,"suffix":3},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1}],"spending":{"currency":{"transmute":1,"augmentation":0,"regal":1,"poe2_alchemy":0,"chaos":0,"exalted":3,"annul":0,"divine":0,"artificer":0,"fracturing":0,"vaal":0},"actions":{"transmute":{"normal":0,"greater":0,"perfect":0},"augmentation":{"normal":0,"greater":0,"perfect":0},"regal":{"normal":0,"greater":0,"perfect":0},"chaos":{"normal":0,"greater":0,"perfect":0},"exalted":{"normal":0,"greater":0,"perfect":0},"poe2_lesser_essence":{"3122":0,"3123":0,"3124":0,"3125":0,"3126":0,"3127":0,"3128":0,"3129":0,"3130":0,"3131":0,"3132":0,"3133":0,"3174":0,"3178":0,"3182":0,"3186":0,"3190":0,"3194":0,"3198":0},"poe2_essence":{"3134":0,"3135":0,"3136":0,"3137":0,"3138":0,"3139":0,"3140":0,"3141":0,"3142":0,"3143":0,"3144":0,"3145":0,"3175":0,"3179":0,"3183":0,"3187":0,"3191":0,"3195":0,"3199":0},"poe2_greater_essence":{"3146":0,"3147":0,"3148":0,"3149":0,"3150":0,"3151":0,"3152":0,"3153":0,"3154":0,"3155":0,"3156":0,"3157":0,"3176":0,"3180":0,"3184":0,"3188":0,"3192":0,"3196":0,"3200":0},"poe2_perfect_essence":{"3158":0,"3159":0,"3160":0,"3161":0,"3162":0,"3163":0,"3164":0,"3165":0,"3166":0,"3167":0,"3168":0,"3169":0,"3170":0,"3171":0,"3172":0,"3173":0,"3177":0,"3181":0,"3185":0,"3189":0,"3193":0,"3197":0,"3201":0},"poe2_desecration":{"ancient_collarbone":0,"ancient_jawbone":0,"ancient_ribs":0,"gnawed_collarbone":0,"gnawed_jawbone":0,"gnawed_ribs":0,"preserved_collarbone":0,"preserved_cranium":0,"preserved_jawbone":0,"preserved_ribs":0,"preserved_spine":0},"poe2_omens":{"abyssal_echoes":0,"blackblooded":0,"blessed":0,"dextral_annulment":0,"dextral_crystallisation":0,"dextral_erasure":0,"dextral_exaltation":0,"dextral_necromancy":0,"greater_exaltation":0,"homogenising_exaltation":0,"homogenising_coronation":0,"liege":0,"light":0,"sinistral_annulment":0,"sinistral_crystallisation":0,"sinistral_erasure":0,"sinistral_exaltation":0,"sinistral_necromancy":0,"sovereign":0,"whittling":0},"poe2_runes":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0,"13":0,"14":0,"15":0,"16":0,"17":0,"18":0,"19":0,"20":0,"21":0,"22":0,"23":0,"24":0,"25":0,"26":0,"27":0,"28":0,"29":0,"30":0,"46":0,"47":0,"48":0,"49":0,"50":0,"51":0,"52":0,"53":0,"54":0,"55":0,"56":0,"57":0,"58":0,"59":0,"60":0,"61":0,"73":0,"74":0,"75":0,"76":0,"77":0,"78":0,"79":0,"80":0,"81":0,"82":0,"83":0,"84":0,"85":0,"86":0,"87":0,"88":0,"89":0,"90":0,"91":0,"92":0,"93":0,"lesser":0,"normal":0,"greater":0,"special":0},"poe2_cores":{"31":0,"32":0,"33":0,"34":0,"35":0,"36":0,"37":0,"38":0,"39":0,"40":0,"41":0,"42":0,"43":0,"44":0,"45":0,"101":0,"102":0,"103":0,"104":0,"105":0,"106":0,"107":0,"108":0,"109":0,"110":0,"111":0,"112":0,"113":0,"114":0,"115":0,"116":0,"117":0,"118":0,"119":0},"poe2_talismans":{"62":0,"63":0,"64":0,"65":0,"66":0,"67":0,"68":0,"69":0,"70":0,"71":0,"72":0,"94":0,"95":0,"96":0,"97":0,"98":0,"99":0,"100":0},"catalyst":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0}}},"opened":{"mp":{"prefix":"5121","suffix":null},"cb":{"prefix":null,"suffix":null},"im":{"eldritch_red":null,"eldritch_blue":null,"corrupted":null,"socket":null}},"catalyst":null}"#;

        let starting_item = CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(
            starting_item,
            &item_provider,
        )?;

        let target_item = CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(
            target_item,
            &item_provider,
        )?;

        let calculator = Calculator::generate_item_matrix(
            starting_item,
            target_item,
            &item_provider,
            &market_info,
            MatrixBuilderPreset::HappyPathMatrixBuilder
                .get_instance()
                .0
                .as_ref(),
        )?;

        let chance_inst = StatisticAnalyzerPathPreset::UniquePathChance.get_instance();

        let best_routes_chance = calculator.calculate_statistics(
            &item_provider,
            &market_info,
            3,
            100_000_000, // 100 MB
            chance_inst.0.as_ref(),
        )?;

        for (analyzer, routes) in vec![(&chance_inst, best_routes_chance)] {
            tracing::warn!("Printing results for '{}'", analyzer.0.get_name());

            for br in routes {
                let out = br.to_pretty_string(
                    &item_provider,
                    &market_info,
                    analyzer.0.as_ref(),
                    &calculator,
                    None,
                );
                tracing::info!("{}", out);
            }
        }

        Ok(())
    }
}
