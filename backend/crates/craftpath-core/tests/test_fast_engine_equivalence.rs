//! Equivalence gate for the fast DAG engine: identical results to the legacy
//! exhaustive collectors on a real (cached CoE data) matrix.
//!
//! The all-path collector (`UniquePathChanceMemoryHeavy` preset) enumerates
//! every route and therefore stays as a permanent oracle even after the
//! legacy unique collector is removed.

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
            item::ItemSnapshot,
            provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
            types::THashMap,
        },
        calc::{
            matrix::presets::matrix_builder_presets::MatrixBuilderPreset,
            statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset,
        },
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
        },
    };

    use crate::init_test_tracing;

    fn load_fixture(
        start_file: &str,
        target_file: &str,
    ) -> Result<(ItemInfoProvider, MarketPriceProvider, ItemSnapshot, ItemSnapshot)> {
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

        let item_jsons =
            retrieve_contents_from_urls_with_cache_unstable_order(item_provider_hm, 60 * 60 * 24)?;
        let economy_jsons =
            retrieve_contents_from_urls_with_cache_unstable_order(economy_provider_hm, 60 * 60)?;

        let provider = CraftOfExileItemInfoProvider::parse_from_json(item_jsons.first().unwrap())?;
        let market = PoeNinjaMarketPriceProviderShim::parse(&economy_jsons)?;

        let start = CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(
            &std::fs::read_to_string(format!(
                "../../python_examples/example_items/{start_file}"
            ))?,
            &provider,
        )?;
        let target = CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(
            &std::fs::read_to_string(format!(
                "../../python_examples/example_items/{target_file}"
            ))?,
            &provider,
        )?;

        Ok((provider, market, start, target))
    }

    // tiny indirection so the import list stays tidy
    struct PoeNinjaMarketPriceProviderShim;
    impl PoeNinjaMarketPriceProviderShim {
        fn parse(jsons: &[String]) -> Result<MarketPriceProvider> {
            craftpath_core::external_api::pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider::parse_from_json_list(jsons)
        }
    }

    /// Small fixture: the exhaustive all-path oracle can enumerate it.
    const ORACLE_PAIR: (&str, &str) =
        ("startitem_good_essence_bow.json", "targetitem_good_essence_bow.json");
    /// Heavy fixture: the all-path enumeration exceeds 4 GB here - only the
    /// fast engine can handle it (showcased in the timing test).
    const HEAVY_PAIR: (&str, &str) = (
        "start_item_magic_1_affix_bow.json",
        "expensive_target_item_rare_6_affix_bow.json",
    );

    fn build_matrix(
        pair: (&str, &str),
    ) -> Result<(Calculator, ItemInfoProvider, MarketPriceProvider)> {
        let (provider, market, start, target) = load_fixture(pair.0, pair.1)?;
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
        Ok((calculator, provider, market))
    }

    fn weights(routes: &[craftpath_core::api::calculator::ItemRoute]) -> Vec<f64> {
        routes.iter().map(|r| *r.weight.get_raw_value()).collect()
    }

    fn assert_weight_multisets_equal(a: &[f64], b: &[f64], label: &str) {
        assert_eq!(a.len(), b.len(), "{label}: length mismatch {} vs {}", a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert!(
                (x - y).abs() <= f64::EPSILON * x.abs().max(1.0) * 4.0,
                "{label}: weight mismatch {x} vs {y}"
            );
        }
    }

    /// Fast chance preset must match the exhaustive all-path oracle's top-K.
    #[test]
    fn test_fast_chance_matches_all_path_oracle() -> Result<()> {
        init_test_tracing();
        let (calculator, provider, market) = build_matrix(ORACLE_PAIR)?;

        for k in [5u32, 50u32] {
            let oracle = calculator.calculate_statistics(
                &provider,
                &market,
                k,
                4_000_000_000,
                StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy
                    .get_instance()
                    .0
                    .as_ref(),
            )?;
            let fast = calculator.calculate_statistics(
                &provider,
                &market,
                k,
                4_000_000_000,
                StatisticAnalyzerPathPreset::UniquePathChance
                    .get_instance()
                    .0
                    .as_ref(),
            )?;

            let mut oracle_top: Vec<f64> = weights(&oracle);
            oracle_top.truncate(k as usize);
            let fast_w = weights(&fast);
            assert_weight_multisets_equal(&oracle_top, &fast_w, &format!("chance k={k}"));
        }
        Ok(())
    }

    /// Cost + efficiency presets: results must be sorted correctly and agree
    /// with a brute-force rescoring of the exhaustive oracle's route set.
    #[test]
    fn test_fast_cost_and_efficiency_against_oracle() -> Result<()> {
        init_test_tracing();
        let (calculator, provider, market) = build_matrix(ORACLE_PAIR)?;

        // exhaustive route universe (sorted by chance, but contains ALL routes)
        let universe = calculator.calculate_statistics(
            &provider,
            &market,
            u32::MAX,
            4_000_000_000,
            StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy
                .get_instance()
                .0
                .as_ref(),
        )?;

        let cost_of = |route: &craftpath_core::api::calculator::ItemRoute| -> f64 {
            use craftpath_core::api::provider::market_prices::PriceKind;
            route
                .route
                .iter()
                .map(|n| {
                    n.currency_list.list.iter().fold(0_f64, |a, b| {
                        a + market.currency_convert(
                            &market.try_lookup_currency_in_divines_default_if_fail(b, &provider),
                            &PriceKind::Exalted,
                        )
                    })
                })
                .sum()
        };
        let efficiency_of = |route: &craftpath_core::api::calculator::ItemRoute| -> f64 {
            let chance = route.chance.get_raw_value();
            let tries =
                ((((1.0_f64 - 0.6).ln() / (1.0_f64 - chance).ln()).ceil()) as u64).max(1);
            cost_of(route) * tries as f64
        };

        let k = 10usize;

        // ---- cost ----
        let mut expected_cost: Vec<f64> = universe.iter().map(|r| cost_of(r)).collect();
        expected_cost.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected_cost.truncate(k);

        let fast_cost = calculator.calculate_statistics(
            &provider,
            &market,
            k as u32,
            4_000_000_000,
            StatisticAnalyzerPathPreset::UniquePathCost
                .get_instance()
                .0
                .as_ref(),
        )?;
        assert_weight_multisets_equal(&expected_cost, &weights(&fast_cost), "cost");

        // ---- efficiency ----
        let mut expected_eff: Vec<f64> = universe.iter().map(|r| efficiency_of(r)).collect();
        expected_eff.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected_eff.truncate(k);

        let fast_eff = calculator.calculate_statistics(
            &provider,
            &market,
            k as u32,
            4_000_000_000,
            StatisticAnalyzerPathPreset::UniquePathEfficiency
                .get_instance()
                .0
                .as_ref(),
        )?;
        assert_weight_multisets_equal(&expected_eff, &weights(&fast_eff), "efficiency");

        Ok(())
    }

    /// Wall-time comparison, excluded from normal runs:
    /// `cargo test -p craftpath-core --test test_fast_engine_equivalence -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn test_timing_comparison() -> Result<()> {
        init_test_tracing();
        let (calculator, provider, market) = build_matrix(ORACLE_PAIR)?;
        let k = 10u32;

        let t0 = std::time::Instant::now();
        let slow = calculator.calculate_statistics(
            &provider,
            &market,
            k,
            4_000_000_000,
            StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy
                .get_instance()
                .0
                .as_ref(),
        )?;
        let slow_elapsed = t0.elapsed();

        let t1 = std::time::Instant::now();
        let fast = calculator.calculate_statistics(
            &provider,
            &market,
            k,
            4_000_000_000,
            StatisticAnalyzerPathPreset::UniquePathChance
                .get_instance()
                .0
                .as_ref(),
        )?;
        let fast_elapsed = t1.elapsed();

        println!(
            "[4-affix] matrix nodes: {} | exhaustive: {:?} ({} routes) | fast DP: {:?} ({} routes) | speedup: {:.1}x",
            calculator.matrix.len(),
            slow_elapsed,
            slow.len(),
            fast_elapsed,
            fast.len(),
            slow_elapsed.as_secs_f64() / fast_elapsed.as_secs_f64().max(1e-9),
        );

        // the heavy fixture: exhaustive enumeration exceeds 4 GB RAM, the
        // fast engine handles it directly
        let (calculator, provider, market) = build_matrix(HEAVY_PAIR)?;
        let t2 = std::time::Instant::now();
        let fast_heavy = calculator.calculate_statistics(
            &provider,
            &market,
            k,
            4_000_000_000,
            StatisticAnalyzerPathPreset::UniquePathChance
                .get_instance()
                .0
                .as_ref(),
        )?;
        println!(
            "[6-affix] matrix nodes: {} | exhaustive: RAM-limit abort (>4 GB) | fast DP: {:?} ({} routes)",
            calculator.matrix.len(),
            t2.elapsed(),
            fast_heavy.len(),
        );
        Ok(())
    }
}
