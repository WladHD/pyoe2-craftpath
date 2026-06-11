pub fn init_test_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt().with_target(false).try_init();
    });
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use craftpath_core::{
        api::{
            calculator::Calculator,
            currency::CraftCurrencyEnum,
            item::ItemSnapshot,
            types::{
                AffixSpecifier, AffixTierConstraints, AffixTierLevel, AffixTierLevelBoundsEnum,
                EssenceKindEnum, ItemLevel, ItemRarityEnum, THashMap, THashSet,
            },
        },
        calc::{
            matrix::presets::matrix_builder_presets::MatrixBuilderPreset,
            statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset,
        },
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
            pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider,
        },
    };

    use crate::init_test_tracing;

    /// Game patch 0.5.0: Alloys (e.g. Transcendent Alloy) apply like Perfect
    /// essences. An empty Rare item with an alloy-granted target mod must be
    /// reachable through an `Essence(<alloy id>)` step.
    #[test]
    fn test_calculator_alloy_crystallisation() -> Result<()> {
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

        let item_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
            item_provider_hm,
            60 * 60 * 24,
        )?;
        let economy_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
            economy_provider_hm,
            60 * 60,
        )?;

        let provider = CraftOfExileItemInfoProvider::parse_from_json(item_jsons.first().unwrap())?;
        let market = PoeNinjaMarketPriceProvider::parse_from_json_list(&economy_jsons)?;

        // alloys must be classified as such
        let alloys: Vec<_> = provider
            .cache_essence_def
            .iter()
            .filter(|(_, def)| def.kind == EssenceKindEnum::Alloy)
            .collect();
        assert!(
            !alloys.is_empty(),
            "expected alloy essences in the CoE data"
        );

        // pick an alloy with a usable (base, affix) entry that is craftable
        // on that base (present in the base's weight table)
        let (alloy_id, base_id, affix_id, min_lvl) = alloys
            .iter()
            .find_map(|(essence_id, def)| {
                def.base_tier_table.iter().find_map(|(base_id, mods)| {
                    mods.iter().find_map(|(affix_id, meta)| {
                        provider
                            .lookup_base_item_mods(base_id)
                            .ok()
                            .map(|_| {
                                (
                                    (*essence_id).clone(),
                                    base_id.clone(),
                                    affix_id.clone(),
                                    meta.min_item_level.clone(),
                                )
                            })
                    })
                })
            })
            .expect("no alloy with a base/affix entry");

        let item_level = ItemLevel::from((*min_lvl.get_raw_value()).max(81));

        let start = ItemSnapshot {
            item_level: item_level.clone(),
            rarity: ItemRarityEnum::Rare,
            base_id: base_id.clone(),
            affixes: THashSet::default(),
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        };

        let mut target = start.clone();
        target.affixes.insert(AffixSpecifier {
            affix: affix_id,
            fractured: false,
            tier: AffixTierConstraints {
                tier: AffixTierLevel::from(1),
                bounds: AffixTierLevelBoundsEnum::Minimum,
            },
        });

        let calculator = Calculator::generate_item_matrix(
            start,
            target,
            &provider,
            &market,
            MatrixBuilderPreset::HappyPathMatrixBuilder
                .get_instance()
                .0
                .as_ref(),
        )?;

        let routes = calculator.calculate_statistics(
            &provider,
            &market,
            5,
            1_000_000_000,
            StatisticAnalyzerPathPreset::UniquePathChance
                .get_instance()
                .0
                .as_ref(),
        )?;

        assert!(!routes.is_empty(), "no route to the alloy target");

        let alloy_route_exists = routes.iter().any(|route| {
            route.route.iter().any(|node| {
                node.currency_list
                    .list
                    .iter()
                    .any(|c| matches!(c, CraftCurrencyEnum::Essence(id) if *id == alloy_id))
            })
        });
        assert!(
            alloy_route_exists,
            "expected a route using Essence({alloy_id:?}); routes: {routes:#?}"
        );

        Ok(())
    }
}
