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
            provider::market_prices::MarketPriceProvider,
            types::{
                AffixLocationEnum, AffixSpecifier, AffixTierConstraints, AffixTierLevel,
                AffixTierLevelBoundsEnum, ItemLevel, ItemRarityEnum, THashMap, THashSet,
            },
        },
        calc::{
            matrix::presets::matrix_builder_presets::MatrixBuilderPreset,
            statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset,
        },
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
        },
    };

    use crate::init_test_tracing;

    /// Game patch 0.5.0: a target with one corruption implicit is reachable
    /// via a final `{VaalOrb}` step with chance 1/4 * w/W (1/3 with Omen of
    /// Corruption), where W is the base's corrupted-implicit pool weight.
    #[test]
    fn test_calculator_vaal_orb_implicit() -> Result<()> {
        init_test_tracing();

        let hm = THashMap::from_iter(vec![(
            "./cache/coe2.json".to_string(),
            "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
        )]);
        let jsons = retrieve_contents_from_urls_with_cache_unstable_order(hm, 60 * 60 * 24)?;
        let provider = CraftOfExileItemInfoProvider::parse_from_json(jsons.first().unwrap())?;

        // pick a base with a corrupted-implicit pool
        let (base_id, implicit_affix_id, pool_size) = provider
            .cache_item_affix_table
            .iter()
            .find_map(|(base_id, table)| {
                let corrupted: Vec<_> = table
                    .keys()
                    .filter(|affix_id| {
                        provider
                            .lookup_affix_definition(affix_id)
                            .map(|def| def.affix_location == AffixLocationEnum::Corrupted)
                            .unwrap_or(false)
                    })
                    .collect();
                (!corrupted.is_empty())
                    .then(|| (base_id.clone(), corrupted[0].clone(), corrupted.len()))
            })
            .expect("no base with corruption implicits");

        tracing::info!(
            "using base {:?} with {} corrupted implicits, target {:?}",
            base_id,
            pool_size,
            implicit_affix_id
        );

        let start = ItemSnapshot {
            item_level: ItemLevel::from(81),
            rarity: ItemRarityEnum::Rare,
            base_id,
            affixes: THashSet::default(),
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        };

        let mut target = start.clone();
        target.affixes.insert(AffixSpecifier {
            affix: implicit_affix_id,
            fractured: false,
            tier: AffixTierConstraints {
                tier: AffixTierLevel::from(1),
                bounds: AffixTierLevelBoundsEnum::Minimum,
            },
        });

        let market = MarketPriceProvider {
            cache_market_prices: THashMap::default(),
            cache_exchange_rate_div_to_exalted: 100.0,
            cache_exchange_rate_div_to_chaos: 1000.0,
        };

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

        assert!(!routes.is_empty(), "no route to the implicit target");

        // best route: single Vaal step (with the omen, since 1/3 > 1/4),
        // chance = branch * uniform pick from the pool
        let best = &routes[0];
        assert_eq!(best.route.len(), 1, "expected a single-step route");
        let step = &best.route[0];
        assert!(
            step.currency_list
                .list
                .iter()
                .any(|c| matches!(c, CraftCurrencyEnum::VaalOrb())),
            "final step must be a Vaal Orb: {:?}",
            step.currency_list
        );

        // all corrupted tier weights are 1 in the CoE data -> uniform pick
        let with_omen = step
            .currency_list
            .list
            .iter()
            .any(|c| matches!(c, CraftCurrencyEnum::OmenOfCorruption()));
        let branch = if with_omen { 1.0 / 3.0 } else { 1.0 / 4.0 };
        let expected = branch / pool_size as f64;
        let actual = step.chance.to_f64();
        assert!(
            (actual - expected).abs() < 1e-9,
            "chance {actual} != expected {expected} (branch {branch}, pool {pool_size})"
        );

        Ok(())
    }
}
