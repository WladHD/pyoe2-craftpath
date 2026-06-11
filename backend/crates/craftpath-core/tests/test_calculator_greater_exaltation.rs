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
            item::{Item, ItemSnapshot},
            types::{
                AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierConstraints,
                AffixTierLevel, AffixTierLevelBoundsEnum, ItemLevel, ItemRarityEnum, THashMap,
                THashSet,
            },
        },
        calc::matrix::happy_path_impl::propagators::exalted_orb::ExaltedOrbPropagator,
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
        },
    };

    use crate::init_test_tracing;

    /// Game patch 0.5.0: Omen of Greater Exaltation makes the next Exalted
    /// Orb add two affixes at once. For a Rare missing two target affixes the
    /// propagator must offer the pair branch, with the unordered
    /// without-replacement pair probability - strictly better than the
    /// product of two sequential single-exalt chances.
    #[test]
    fn test_greater_exaltation_pair_branch() -> Result<()> {
        init_test_tracing();

        let hm = THashMap::from_iter(vec![(
            "./cache/coe2.json".to_string(),
            "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
        )]);
        let jsons = retrieve_contents_from_urls_with_cache_unstable_order(hm, 60 * 60 * 24)?;
        let provider = CraftOfExileItemInfoProvider::parse_from_json(jsons.first().unwrap())?;

        // pick any base with at least 3 Base-class prefix/suffix mods, use
        // one as the existing mod (rare needs >= 1 affix) and two as targets
        let (base_id, mods) = provider
            .cache_item_affix_table
            .iter()
            .find_map(|(base_id, table)| {
                // greedily collect 3 Base-class prefix/suffix mods with
                // pairwise-disjoint exclusive groups (so nothing blocks)
                let mut picked: Vec<craftpath_core::api::types::AffixId> = Vec::new();
                let mut seen_groups: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for affix_id in table.keys() {
                    let Ok(def) = provider.lookup_affix_definition(affix_id) else {
                        continue;
                    };
                    if def.affix_class != AffixClassEnum::Base
                        || !matches!(
                            def.affix_location,
                            AffixLocationEnum::Prefix | AffixLocationEnum::Suffix
                        )
                        || def.exlusive_groups.iter().any(|g| seen_groups.contains(g))
                    {
                        continue;
                    }
                    seen_groups.extend(def.exlusive_groups.iter().cloned());
                    picked.push(affix_id.clone());
                    if picked.len() == 3 {
                        break;
                    }
                }
                (picked.len() == 3).then(|| (base_id.clone(), picked))
            })
            .expect("no base with three independent mods");

        let spec = |affix_id, tier| AffixSpecifier {
            affix: affix_id,
            fractured: false,
            tier: AffixTierConstraints {
                tier: AffixTierLevel::from(tier),
                bounds: AffixTierLevelBoundsEnum::Minimum,
            },
        };

        let mut start_affixes: THashSet<AffixSpecifier> = THashSet::default();
        start_affixes.insert(spec(mods[0].clone(), 1));

        let start = ItemSnapshot {
            item_level: ItemLevel::from(81),
            rarity: ItemRarityEnum::Rare,
            base_id,
            affixes: start_affixes,
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        };

        let mut target = start.clone();
        target.affixes.insert(spec(mods[1].clone(), 1));
        target.affixes.insert(spec(mods[2].clone(), 1));

        let item = Item::build_with(start.clone(), &target, &provider)?;
        let propagated = ExaltedOrbPropagator::propagate_step_default(&item, &target, &provider)?;

        // the Greater-Exaltation branch must exist ...
        let greater_branch = propagated
            .iter()
            .find(|(currency_list, _)| {
                currency_list
                    .list
                    .contains(&CraftCurrencyEnum::OmenOfGreaterExaltation())
                    && currency_list
                        .list
                        .contains(&CraftCurrencyEnum::ExaltedOrbNormal())
            })
            .expect("no Greater Exaltation branch was propagated");

        // ... and contain an outcome holding BOTH missing target affixes
        let pair_outcome = greater_branch
            .1
            .iter()
            .find(|outcome| {
                outcome.next.affixes.iter().any(|a| a.affix == mods[1])
                    && outcome.next.affixes.iter().any(|a| a.affix == mods[2])
            })
            .expect("no outcome with both target affixes");

        // sanity: pair chance is a real probability and beats the product of
        // the two sequential single-exalt chances (no wasted second draw)
        let plain_branch = propagated
            .iter()
            .find(|(currency_list, _)| {
                currency_list.list.len() == 1
                    && currency_list
                        .list
                        .contains(&CraftCurrencyEnum::ExaltedOrbNormal())
            })
            .expect("no plain exalt branch");

        let single = |affix_id: &craftpath_core::api::types::AffixId| -> f64 {
            plain_branch
                .1
                .iter()
                .find(|o| o.next.affixes.iter().any(|a| &a.affix == affix_id))
                .map(|o| o.chance.to_f64())
                .unwrap_or(0.0)
        };

        let pair_chance = pair_outcome.chance.to_f64();
        let sequential = single(&mods[1]) * single(&mods[2]);

        assert!(pair_chance > 0.0 && pair_chance <= 1.0);
        assert!(
            pair_chance > sequential,
            "pair chance {pair_chance} should beat sequential product {sequential}"
        );

        // Calculator end-to-end: target reachable
        let market = craftpath_core::api::provider::market_prices::MarketPriceProvider {
            cache_market_prices: THashMap::default(),
            cache_exchange_rate_div_to_exalted: 100.0,
            cache_exchange_rate_div_to_chaos: 1000.0,
        };
        let calc = Calculator::generate_item_matrix(
            start,
            target,
            &provider,
            &market,
            craftpath_core::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset::HappyPathMatrixBuilder
                .get_instance()
                .0
                .as_ref(),
        );
        assert!(calc.is_ok(), "matrix generation failed: {:?}", calc.err());

        Ok(())
    }
}
