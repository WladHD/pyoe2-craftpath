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
            currency::CraftCurrencyEnum,
            item::{Item, ItemSnapshot},
            matrix_propagator::MatrixPropagator,
            types::{
                AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierConstraints,
                AffixTierLevel, AffixTierLevelBoundsEnum, ItemLevel, ItemRarityEnum, THashMap,
                THashSet,
            },
        },
        calc::matrix::happy_path_impl::propagators::orb_of_annulment::OrbOfAnnulmentPropagator,
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
        },
    };

    use crate::init_test_tracing;

    /// MECHANICS.md V5/V9: desecrated modifiers act as removable blockers.
    /// An unwanted desecrated mod must be removable by the Orb of Annulment,
    /// and `{OrbOfAnnulment, OmenOfLight}` must remove it with certainty
    /// (the omen restricts the removal pool to desecrated mods only).
    #[test]
    fn test_desecrated_blocker_removal() -> Result<()> {
        init_test_tracing();

        let hm = THashMap::from_iter(vec![(
            "./cache/coe2.json".to_string(),
            "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
        )]);
        let jsons = retrieve_contents_from_urls_with_cache_unstable_order(hm, 60 * 60 * 24)?;
        let provider = CraftOfExileItemInfoProvider::parse_from_json(jsons.first().unwrap())?;

        // find a base carrying both a desecrated-class mod and a normal mod
        let (base_id, desecrated_id, normal_id) = provider
            .cache_item_affix_table
            .iter()
            .find_map(|(base_id, table)| {
                let mut desecrated = None;
                let mut normal = None;
                for affix_id in table.keys() {
                    let Ok(def) = provider.lookup_affix_definition(affix_id) else {
                        continue;
                    };
                    if !matches!(
                        def.affix_location,
                        AffixLocationEnum::Prefix | AffixLocationEnum::Suffix
                    ) {
                        continue;
                    }
                    match def.affix_class {
                        AffixClassEnum::Desecrated if desecrated.is_none() => {
                            desecrated = Some(affix_id.clone())
                        }
                        AffixClassEnum::Base if normal.is_none() => {
                            normal = Some(affix_id.clone())
                        }
                        _ => {}
                    }
                    if desecrated.is_some() && normal.is_some() {
                        break;
                    }
                }
                match (desecrated, normal) {
                    (Some(d), Some(n)) => Some((base_id.clone(), d, n)),
                    _ => None,
                }
            })
            .expect("no base with both desecrated and normal mods");

        let spec = |affix_id| AffixSpecifier {
            affix: affix_id,
            fractured: false,
            tier: AffixTierConstraints {
                tier: AffixTierLevel::from(1),
                bounds: AffixTierLevelBoundsEnum::Minimum,
            },
        };

        // rare item with one normal (wanted) and one desecrated (unwanted,
        // i.e. the blocker that has done its job) mod
        let mut affixes: THashSet<AffixSpecifier> = THashSet::default();
        affixes.insert(spec(normal_id.clone()));
        affixes.insert(spec(desecrated_id.clone()));

        let start = ItemSnapshot {
            item_level: ItemLevel::from(81),
            rarity: ItemRarityEnum::Rare,
            base_id,
            affixes,
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        };

        let mut target = start.clone();
        target.affixes.retain(|a| a.affix != desecrated_id);

        let item = Item::build_with(start, &target, &provider)?;
        assert!(
            item.helper
                .unwanted_affixes
                .iter()
                .any(|a| a.affix == desecrated_id),
            "the desecrated mod must be classified as unwanted"
        );

        let propagator = OrbOfAnnulmentPropagator;
        assert!(propagator.is_applicable(&item, &provider));
        let propagated = propagator.propagate_step(&item, &target, &provider)?;

        // plain annulment: removes the blocker with 1/2 (two removable mods)
        let plain = propagated
            .iter()
            .find(|(list, _)| list.list.len() == 1)
            .expect("no plain annulment branch");
        let plain_outcome = plain
            .1
            .iter()
            .find(|o| !o.next.affixes.iter().any(|a| a.affix == desecrated_id))
            .expect("plain annulment cannot remove the desecrated blocker");
        assert!((plain_outcome.chance.to_f64() - 0.5).abs() < 1e-9);

        // Omen of Light: pool restricted to desecrated mods -> certainty
        let light = propagated
            .iter()
            .find(|(list, _)| list.list.contains(&CraftCurrencyEnum::OmenOfLight()))
            .expect("no Omen of Light branch");
        let light_outcome = light
            .1
            .iter()
            .find(|o| !o.next.affixes.iter().any(|a| a.affix == desecrated_id))
            .expect("Omen of Light branch does not remove the desecrated mod");
        assert!(
            (light_outcome.chance.to_f64() - 1.0).abs() < 1e-9,
            "Omen of Light must remove the only desecrated mod with certainty, got {}",
            light_outcome.chance.to_f64()
        );

        Ok(())
    }
}
