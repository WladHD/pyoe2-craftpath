use anyhow::{Result, anyhow};

use crate::{
    api::{
        item::ItemSnapshot,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixId, AffixSpecifier, AffixTierConstraints, AffixTierLevel,
            AffixTierLevelBoundsEnum, BaseItemId, ItemLevel, ItemRarityEnum, THashSet,
        },
    },
    external_api::coe_emulator::coe_emulator_item_snapshot_definition::EmulatorItemExport,
};

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
pub struct CraftOfExileEmulatorItemImport;

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::prelude::pymethods)]
impl CraftOfExileEmulatorItemImport {
    #[pyo3(name = "parse_itemsnapshot_from_string")]
    #[staticmethod]
    pub fn parse_itemsnapshot_from_string_py(
        item_json: &str,
        provider: &ItemInfoProvider,
    ) -> pyo3::PyResult<ItemSnapshot> {
        Self::parse_itemsnapshot_from_string(item_json, provider)
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }
}

impl CraftOfExileEmulatorItemImport {
    pub fn parse_itemsnapshot_from_string(
        item_json: &str,
        provider: &ItemInfoProvider,
    ) -> Result<ItemSnapshot> {
        let parsed: EmulatorItemExport = serde_json::from_str(&item_json)?;

        let item_base = BaseItemId::from(parsed.settings.base);
        let item_def = provider.lookup_base_item_mods(&item_base)?;

        let mut affix_hm = THashSet::<AffixSpecifier>::default();

        parsed
            .data
            .iaffixes
            .iter()
            .filter_map(|e| {
                let affix_id = AffixId::from(e.id);

                let Some(af) = item_def.get(&affix_id) else {
                    return None;
                };

                Some(AffixSpecifier {
                    affix: affix_id,
                    tier: AffixTierConstraints {
                        tier: AffixTierLevel::from((af.len() as u8) - e.tindex),
                        bounds: AffixTierLevelBoundsEnum::Minimum,
                    },
                    fractured: e.frac,
                })
            })
            .for_each(|e| {
                affix_hm.insert(e);
            });

        Ok(ItemSnapshot {
            base_id: item_base,
            item_level: ItemLevel::from(parsed.settings.ilvl),
            rarity: match parsed.settings.rarity.to_lowercase().as_ref() {
                "rare" => ItemRarityEnum::Rare,
                "magic" => ItemRarityEnum::Magic,
                "normal" => ItemRarityEnum::Normal,
                "unique" => ItemRarityEnum::Unique,
                _ => return Err(anyhow!("Rarity {} not implemented", parsed.settings.rarity)),
            },
            affixes: affix_hm,
            corrupted: parsed.settings.corrupted,
            sockets: THashSet::default(), // TODO parse actual sockets
            allowed_sockets: parsed.settings.sockets,
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing::instrument;

    use crate::{
        api::types::{AffixId, AffixTierLevel, AffixTierLevelBoundsEnum, THashMap},
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
        },
        utils::logger_utils::init_tracing,
    };

    #[test]
    #[instrument]
    fn test_parse_itemsnapshot_from_string() -> Result<()> {
        init_tracing();

        let raw_item = r#"{"settings":{"bgroup":7,"base":20,"bitem":231,"ilvl":100,"rarity":"rare","influences":null,"sockets":1,"socketed":[],"quality":20,"exmods":null,"corrupted":1},"params":{"mode":null,"currency":null,"action":null,"subaction":null,"ssaction":null,"disabled":"|catalyst|poe2_desecrationancient_ribs|poe2_desecrationgnawed_ribs|poe2_desecrationpreserved_ribs|poe2_desecrationancient_collarbone|poe2_desecrationgnawed_collarbone|poe2_desecrationpreserved_collarbone|poe2_desecrationpreserved_cranium|poe2_desecrationpreserved_spine|poe2_desecrationgnawed_collarbone|poe2_desecrationgnawed_ribs|poe2_desecrationgnawed_jawbone|fossil18|eldritch|vendor|","cursor":"url('images/ui_poe2/method_vaal.png') 39 39, default"},"data":{"implicits":null,"iaffixes":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":3,"bench":0,"frac":0,"maven":0,"nvalues":"[[35,44],[73,97]]","rolls":[38,88],"weight":"1000"},{"atype":"prefix","id":5120,"mgrp":"1","modgroups":["LocalPhysicalDamagePercent"],"tindex":1,"bench":0,"frac":0,"maven":0,"nvalues":"[[50,64]]","rolls":[53],"weight":"1000"},{"id":"5053","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["LifeGainedFromEnemyDeath"],"weight":750,"nvalues":"[[54,68]]","amgs":null,"tags":"|1|","rolls":[60],"bench":0,"frac":1,"maven":0,"chance":0.01661037594817563},{"id":"5056","mgrp":"1","tindex":4,"atype":"prefix","modgroups":["IncreasedAccuracy"],"weight":800,"nvalues":"[[124,167]]","amgs":null,"tags":"|3|","rolls":[129],"bench":0,"frac":0,"maven":0,"chance":0.03084040092521203}],"iaffbt":{"prefix":3,"suffix":1},"imprint":null,"eldritch":null,"meta_flags":{}},"log":[{"mode":"forced","currency":null,"action":"mp","ssaction":3,"subaction":5121,"forced":"add","add":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":3,"bench":0,"frac":0,"maven":0,"nvalues":"[[35,44],[73,97]]","rolls":[38,88],"weight":"1000"}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":231,"base":20,"bgroup":7,"rarity":"normal","affixes":[],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":0,"suffix":0},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"forced","currency":null,"action":"mp","ssaction":1,"subaction":5120,"forced":"add","add":[{"atype":"prefix","id":5120,"mgrp":"1","modgroups":["LocalPhysicalDamagePercent"],"tindex":1,"bench":0,"frac":0,"maven":0,"nvalues":"[[50,64]]","rolls":[53],"weight":"1000"}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":231,"base":20,"bgroup":7,"rarity":"magic","affixes":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":3,"bench":0,"frac":0,"maven":0,"nvalues":"[[35,44],[73,97]]","rolls":[38,88],"weight":"1000"}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":1,"suffix":0},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"exalted","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":[{"id":"5053","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["LifeGainedFromEnemyDeath"],"weight":750,"nvalues":"[[54,68]]","amgs":null,"tags":"|1|","rolls":[60],"bench":0,"frac":0,"maven":0,"chance":0.01661037594817563}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":231,"base":20,"bgroup":7,"rarity":"rare","affixes":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":3,"bench":0,"frac":0,"maven":0,"nvalues":"[[35,44],[73,97]]","rolls":[38,88],"weight":"1000"},{"atype":"prefix","id":5120,"mgrp":"1","modgroups":["LocalPhysicalDamagePercent"],"tindex":1,"bench":0,"frac":0,"maven":0,"nvalues":"[[50,64]]","rolls":[53],"weight":"1000"}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":2,"suffix":0},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"exalted","action":null,"ssaction":null,"subaction":"normal","forced":null,"add":[{"id":"5056","mgrp":"1","tindex":4,"atype":"prefix","modgroups":["IncreasedAccuracy"],"weight":800,"nvalues":"[[124,167]]","amgs":null,"tags":"|3|","rolls":[129],"bench":0,"frac":0,"maven":0,"chance":0.03084040092521203}],"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":231,"base":20,"bgroup":7,"rarity":"rare","affixes":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":3,"bench":0,"frac":0,"maven":0,"nvalues":"[[35,44],[73,97]]","rolls":[38,88],"weight":"1000"},{"atype":"prefix","id":5120,"mgrp":"1","modgroups":["LocalPhysicalDamagePercent"],"tindex":1,"bench":0,"frac":0,"maven":0,"nvalues":"[[50,64]]","rolls":[53],"weight":"1000"},{"id":"5053","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["LifeGainedFromEnemyDeath"],"weight":750,"nvalues":"[[54,68]]","amgs":null,"tags":"|1|","rolls":[60],"bench":0,"frac":0,"maven":0,"chance":0.01661037594817563}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":2,"suffix":1},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"fracturing","action":null,"ssaction":null,"subaction":null,"forced":null,"add":null,"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":231,"base":20,"bgroup":7,"rarity":"rare","affixes":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":3,"bench":0,"frac":0,"maven":0,"nvalues":"[[35,44],[73,97]]","rolls":[38,88],"weight":"1000"},{"atype":"prefix","id":5120,"mgrp":"1","modgroups":["LocalPhysicalDamagePercent"],"tindex":1,"bench":0,"frac":0,"maven":0,"nvalues":"[[50,64]]","rolls":[53],"weight":"1000"},{"id":"5053","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["LifeGainedFromEnemyDeath"],"weight":750,"nvalues":"[[54,68]]","amgs":null,"tags":"|1|","rolls":[60],"bench":0,"frac":0,"maven":0,"chance":0.01661037594817563},{"id":"5056","mgrp":"1","tindex":4,"atype":"prefix","modgroups":["IncreasedAccuracy"],"weight":800,"nvalues":"[[124,167]]","amgs":null,"tags":"|3|","rolls":[129],"bench":0,"frac":0,"maven":0,"chance":0.03084040092521203}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":3,"suffix":1},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1},{"mode":"currency","currency":"vaal","action":null,"ssaction":null,"subaction":null,"forced":null,"add":null,"omens":null,"rem":null,"upg":null,"sts":null,"det":null,"ilvl":100,"bitem":231,"base":20,"bgroup":7,"rarity":"rare","affixes":[{"atype":"prefix","id":5121,"mgrp":"1","modgroups":["LocalIncreasedPhysicalDamagePercentAndAccuracyRating"],"tindex":3,"bench":0,"frac":0,"maven":0,"nvalues":"[[35,44],[73,97]]","rolls":[38,88],"weight":"1000"},{"atype":"prefix","id":5120,"mgrp":"1","modgroups":["LocalPhysicalDamagePercent"],"tindex":1,"bench":0,"frac":0,"maven":0,"nvalues":"[[50,64]]","rolls":[53],"weight":"1000"},{"id":"5053","mgrp":"1","tindex":6,"atype":"suffix","modgroups":["LifeGainedFromEnemyDeath"],"weight":750,"nvalues":"[[54,68]]","amgs":null,"tags":"|1|","rolls":[60],"bench":0,"frac":1,"maven":0,"chance":0.01661037594817563},{"id":"5056","mgrp":"1","tindex":4,"atype":"prefix","modgroups":["IncreasedAccuracy"],"weight":800,"nvalues":"[[124,167]]","amgs":null,"tags":"|3|","rolls":[129],"bench":0,"frac":0,"maven":0,"chance":0.03084040092521203}],"implicits":null,"eldritch":null,"influences":null,"imprint":null,"psn":{"prefix":3,"suffix":1},"catalyst":null,"sockets":0,"socketed":[],"quality":20,"nums":1}],"spending":{"currency":{"transmute":0,"augmentation":0,"regal":0,"poe2_alchemy":0,"chaos":0,"exalted":2,"annul":0,"divine":0,"artificer":0,"fracturing":1,"vaal":1},"actions":{"transmute":{"normal":0,"greater":0,"perfect":0},"augmentation":{"normal":0,"greater":0,"perfect":0},"regal":{"normal":0,"greater":0,"perfect":0},"chaos":{"normal":0,"greater":0,"perfect":0},"exalted":{"normal":0,"greater":0,"perfect":0},"poe2_lesser_essence":{"3122":0,"3123":0,"3124":0,"3125":0,"3126":0,"3127":0,"3128":0,"3129":0,"3130":0,"3131":0,"3132":0,"3133":0,"3174":0,"3178":0,"3182":0,"3186":0,"3190":0,"3194":0,"3198":0},"poe2_essence":{"3134":0,"3135":0,"3136":0,"3137":0,"3138":0,"3139":0,"3140":0,"3141":0,"3142":0,"3143":0,"3144":0,"3145":0,"3175":0,"3179":0,"3183":0,"3187":0,"3191":0,"3195":0,"3199":0},"poe2_greater_essence":{"3146":0,"3147":0,"3148":0,"3149":0,"3150":0,"3151":0,"3152":0,"3153":0,"3154":0,"3155":0,"3156":0,"3157":0,"3176":0,"3180":0,"3184":0,"3188":0,"3192":0,"3196":0,"3200":0},"poe2_perfect_essence":{"3158":0,"3159":0,"3160":0,"3161":0,"3162":0,"3163":0,"3164":0,"3165":0,"3166":0,"3167":0,"3168":0,"3169":0,"3170":0,"3171":0,"3172":0,"3173":0,"3177":0,"3181":0,"3185":0,"3189":0,"3193":0,"3197":0,"3201":0},"poe2_desecration":{"ancient_collarbone":0,"ancient_jawbone":0,"ancient_ribs":0,"gnawed_collarbone":0,"gnawed_jawbone":0,"gnawed_ribs":0,"preserved_collarbone":0,"preserved_cranium":0,"preserved_jawbone":0,"preserved_ribs":0,"preserved_spine":0},"poe2_omens":{"abyssal_echoes":0,"blackblooded":0,"blessed":0,"dextral_annulment":0,"dextral_crystallisation":0,"dextral_erasure":0,"dextral_exaltation":0,"dextral_necromancy":0,"greater_exaltation":0,"homogenising_exaltation":0,"homogenising_coronation":0,"liege":0,"light":0,"sinistral_annulment":0,"sinistral_crystallisation":0,"sinistral_erasure":0,"sinistral_exaltation":0,"sinistral_necromancy":0,"sovereign":0,"whittling":0},"poe2_runes":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0,"13":0,"14":0,"15":0,"16":0,"17":0,"18":0,"19":0,"20":0,"21":0,"22":0,"23":0,"24":0,"25":0,"26":0,"27":0,"28":0,"29":0,"30":0,"46":0,"47":0,"48":0,"49":0,"50":0,"51":0,"52":0,"53":0,"54":0,"55":0,"56":0,"57":0,"58":0,"59":0,"60":0,"61":0,"73":0,"74":0,"75":0,"76":0,"77":0,"78":0,"79":0,"80":0,"81":0,"82":0,"83":0,"84":0,"85":0,"86":0,"87":0,"88":0,"89":0,"90":0,"91":0,"92":0,"93":0,"lesser":0,"normal":0,"greater":0,"special":0},"poe2_cores":{"31":0,"32":0,"33":0,"34":0,"35":0,"36":0,"37":0,"38":0,"39":0,"40":0,"41":0,"42":0,"43":0,"44":0,"45":0,"101":0,"102":0,"103":0,"104":0,"105":0,"106":0,"107":0,"108":0,"109":0,"110":0,"111":0,"112":0,"113":0,"114":0,"115":0,"116":0,"117":0,"118":0,"119":0},"poe2_talismans":{"62":0,"63":0,"64":0,"65":0,"66":0,"67":0,"68":0,"69":0,"70":0,"71":0,"72":0,"94":0,"95":0,"96":0,"97":0,"98":0,"99":0,"100":0},"catalyst":{"1":0,"2":0,"3":0,"4":0,"5":0,"6":0,"7":0,"8":0,"9":0,"10":0,"11":0,"12":0}}},"opened":{"mp":{"prefix":"5120","suffix":null},"cb":{"prefix":null,"suffix":null},"im":{"eldritch_red":null,"eldritch_blue":null,"corrupted":null,"socket":null}},"catalyst":null}"#;

        let hm = THashMap::from_iter(
            vec![(
                "./cache/coe2.json".to_string(),
                "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
            )]
            .into_iter(),
        );

        let provider = retrieve_contents_from_urls_with_cache_unstable_order(hm, 60_u64 * 60_u64)?;
        let provider = CraftOfExileItemInfoProvider::parse_from_json(
            provider.first().expect("Provider returned no item info"),
        )?;

        let res =
            CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(raw_item, &provider)?;

        assert_eq!(res.affixes.len(), 4);

        assert_eq!(res.allowed_sockets, 1);

        assert_eq!(res.corrupted, true);

        assert_eq!(
            res.affixes
                .iter()
                .any(|test| test.affix == AffixId::from(5121)
                    && test.tier.tier == AffixTierLevel::from(5)
                    && test.tier.bounds == AffixTierLevelBoundsEnum::Minimum),
            true
        );

        assert_eq!(
            res.affixes
                .iter()
                .any(|test| test.affix == AffixId::from(5053)
                    && test.tier.tier == AffixTierLevel::from(2)
                    && test.tier.bounds == AffixTierLevelBoundsEnum::Minimum
                    && test.fractured),
            true
        );

        tracing::info!("Resulting snapshot: {:#?}", res);

        Ok(())
    }
}
