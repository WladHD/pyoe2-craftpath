use num_format::{Locale, ToFormattedString};

#[cfg(feature = "python")]
use crate::api::calculator::DynStatisticAnalyzerPaths;
use crate::{
    api::{
        calculator::{self, Calculator, GroupRoute, ItemRoute, StatisticAnalyzerPaths},
        currency::CraftCurrencyList,
        provider::{
            item_info::ItemInfoProvider,
            market_prices::{MarketPriceProvider, PriceInDivines, PriceKind},
        },
        types::{
            AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierLevelBoundsEnum,
            BaseItemId, THashSet,
        },
    },
    calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset,
    utils::fraction_utils::Fraction,
};
use std::fmt::Write;

impl ItemRoute {
    pub fn locate_group<'a>(
        &self,
        calculated_groups: &'a Vec<GroupRoute>,
    ) -> Option<&'a GroupRoute> {
        let curr = self
            .route
            .iter()
            .map(|e| e.currency_list.clone())
            .collect::<Vec<CraftCurrencyList>>();

        let found = calculated_groups
            .iter()
            .find(|test| test.group.as_slice() == curr.as_slice());

        found
    }

    pub fn to_pretty_string(
        &self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        unique_path_statistic_analyzer: &dyn StatisticAnalyzerPaths,
        calculator: &Calculator,
        calculated_groups: Option<&Vec<GroupRoute>>,
    ) -> String {
        let mut out = String::new();

        if let Some(group) = calculated_groups {
            let found = self.locate_group(&group);

            match found {
                Some(e) => writeln!(
                    &mut out,
                    "{}",
                    e.to_pretty_string(
                        &item_provider,
                        &market_provider,
                        StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance
                            .get_instance()
                            .0
                            .as_ref()
                    )
                )
                .unwrap(),
                None => writeln!(&mut out, "Group info could not be parsed.").unwrap(),
            };
        }

        let start_item = &calculator.starting_item;

        write!(
            &mut out,
            "{}",
            start_item.to_pretty_string(&item_provider, false)
        )
        .unwrap();

        let cost_per_1 = unique_path_statistic_analyzer.calculate_cost_per_craft(
            &self
                .route
                .iter()
                .map(|e| e.currency_list.clone())
                .collect::<Vec<CraftCurrencyList>>(),
            &item_provider,
            &market_provider,
        );

        let tries_for_60 =
            unique_path_statistic_analyzer.calculate_tries_needed_for_60_percent(&self);
        let cost_per_60 =
            PriceInDivines::new((tries_for_60 as f64) * cost_per_1.get_divine_value());

        writeln!(
            &mut out,
            "Exact Chance: {:.5}% | Tries needed for 60%: {} | Cost per Craft: {} | Cost for 60%: {}{}",
            (*self.chance.get_raw_value()) * 100_f64,
            tries_for_60.to_formatted_string(&Locale::en),
            format!(
                "{} EX",
                (market_provider
                    .currency_convert(&cost_per_1, &PriceKind::Exalted)
                    .ceil() as u64)
                    .to_formatted_string(&Locale::en)
            ),
            format!(
                "{} EX",
                (market_provider
                    .currency_convert(&cost_per_60, &PriceKind::Exalted)
                    .ceil() as u64)
                    .to_formatted_string(&Locale::en)
            ),
            match unique_path_statistic_analyzer.format_display_more_info(
                &self,
                &item_provider,
                &market_provider
            ) {
                Some(e) => e,
                None => "".to_string(),
            }
        )
        .unwrap();

        writeln!(
            out,
            "0. Starting with ...{}",
            if start_item.affixes.is_empty() {
                " nothing :3".to_string()
            } else {
                "".to_string()
            }
        )
        .unwrap();

        for affix in &start_item.affixes {
            print_affix(
                &mut out,
                Some(0),
                affix,
                None,
                item_provider,
                true,
                &calculator.starting_item.base_id,
                false,
            );
        }

        let mut prev_affixes = start_item.affixes.clone();
        let mut prev_rarity = start_item.rarity.clone();
        let mut temporary_affixes: THashSet<AffixSpecifier> = THashSet::default();

        for (i, path) in self.route.iter().enumerate() {
            let item: &calculator::ItemMatrixNode =
                calculator.matrix.get(&path.item_matrix_id).unwrap();
            let new_affixes = &item.item.snapshot.affixes;
            let new_rarity = &item.item.snapshot.rarity;

            let added: THashSet<_> = new_affixes.difference(&prev_affixes).collect();
            let removed: THashSet<_> = prev_affixes.difference(&new_affixes).collect();
            let temporary = item.item.meta.mark_for_essence_only;

            writeln!(
                out,
                "{}. Apply {}{}",
                i + 1,
                path.currency_list
                    .list
                    .iter()
                    .map(|e| {
                        let currency_value = market_provider
                            .try_lookup_currency_in_divines_default_if_fail(&e, &item_provider);
                        let currency_value_ex = market_provider
                            .currency_convert(&currency_value, &PriceKind::Exalted)
                            .ceil() as u32;

                        format!(
                            "{} ({} EX)",
                            e.get_item_name(&item_provider),
                            currency_value_ex.to_formatted_string(&Locale::en)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(" + "),
                match temporary {
                    true => " [TEMP]",
                    false => "",
                }
            )
            .unwrap();

            for affix in removed.iter() {
                print_affix(
                    &mut out,
                    Some(i + 1),
                    affix,
                    Some(path.chance),
                    item_provider,
                    false,
                    &calculator.starting_item.base_id,
                    temporary_affixes.contains(&affix),
                );

                temporary_affixes.remove(affix);
            }

            for affix in added.iter() {
                if temporary {
                    temporary_affixes.insert((*affix).clone());
                }

                print_affix(
                    &mut out,
                    Some(i + 1),
                    affix,
                    Some(path.chance),
                    item_provider,
                    true,
                    &calculator.starting_item.base_id,
                    temporary,
                );
            }

            if start_item.allowed_sockets != item.item.snapshot.allowed_sockets {
                print_socket_change(
                    &mut out,
                    i + 1,
                    start_item
                        .allowed_sockets
                        .abs_diff(item.item.snapshot.allowed_sockets),
                    Some(path.chance),
                    item_provider,
                    &calculator.starting_item.base_id,
                );
            }

            if new_rarity != &prev_rarity {
                writeln!(
                    &mut out,
                    "{}. \t! Rarity {:?} -> {:?}",
                    i + 1,
                    prev_rarity,
                    new_rarity
                )
                .unwrap();
            }

            if item.item.snapshot.corrupted {
                writeln!(
                    &mut out,
                    "{}. \t! Corrupted - ensure maximum quality and wanted affixes prior to applying a Vaal Orb, since corrupted items CAN NOT be modified further.",
                    i + 1
                )
                .unwrap();
            }

            prev_affixes = new_affixes.clone();
            prev_rarity = new_rarity.clone();
        }

        out
    }
}

pub fn print_socket_change(
    out: &mut String,
    index: usize,
    current_sockets: u8,
    chance: Option<Fraction>,
    item_provider: &ItemInfoProvider,
    base_id: &BaseItemId,
) {
    let bg = item_provider.lookup_base_group(base_id).unwrap();
    let bg = item_provider.lookup_base_group_definition(&bg).unwrap();

    writeln!(
        out,
        "{}.\t{}{} Socket ({}/{})",
        index,
        if index == 0 { "" } else { "+ " },
        match chance {
            Some(c) => format!("[{} (~{:.3}%)]", c, c.to_f64() * 100_f64).to_string(),
            None => "".to_string(),
        },
        current_sockets,
        bg.max_sockets
    )
    .unwrap();
}

pub fn print_affix(
    out: &mut String,
    index: Option<usize>,
    affix: &AffixSpecifier,
    chance: Option<Fraction>,
    item_provider: &ItemInfoProvider,
    is_added: bool,
    base_id: &BaseItemId,
    is_temporary: bool,
) {
    let affix_def = item_provider.lookup_affix_definition(&affix.affix).unwrap();

    let name = &affix_def.description_template;

    let min_ivl = match affix_def.affix_class {
        AffixClassEnum::Base | AffixClassEnum::Desecrated | AffixClassEnum::Essence => {
            let ilvl = &item_provider
                .lookup_base_item_mods(&base_id)
                .unwrap()
                .get(&affix.affix)
                .unwrap()
                .iter()
                .find(|e| e.0 == &affix.tier.tier)
                .unwrap()
                .1
                .min_item_level;

            format!("ilvl {}", ilvl.get_raw_value()).to_string()
        }
    };

    let more_meta: Vec<Option<String>> = vec![
        match is_temporary {
            true => None,
            false => Some(
                format!(
                    "Tier {}{}",
                    affix.tier.tier.get_raw_value(),
                    match affix.tier.bounds {
                        AffixTierLevelBoundsEnum::Exact => "=",
                        AffixTierLevelBoundsEnum::Minimum => "+",
                    }
                )
                .to_string(),
            ),
        },
        match is_temporary {
            true => None,
            false => Some(min_ivl),
        },
        match affix_def.affix_location {
            AffixLocationEnum::Prefix => Some("Prefix".to_string()),
            AffixLocationEnum::Suffix => Some("Suffix".to_string()),
            _ => None,
        },
        match affix.fractured {
            true => Some("FRAC".to_string()),
            false => None,
        },
        match affix_def.affix_class {
            AffixClassEnum::Base => None,
            AffixClassEnum::Desecrated => Some("Des.".to_string()),
            AffixClassEnum::Essence => Some("Ess.".to_string()),
        },
    ];

    let more_meta = more_meta
        .iter()
        .filter_map(|test| test.clone())
        .collect::<Vec<String>>();

    writeln!(
        out,
        "{}{}[{}{}] '{}'",
        match index {
            Some(i) => format!("{}.\t", i),
            None => "".to_string(),
        },
        match index {
            Some(i) if i != 0 && is_added => "+ ",
            Some(i) if i != 0 && !is_added => "- ",
            _ => "",
        },
        match chance {
            Some(c) => format!("{} (~{:.3}%), ", c, c.to_f64() * 100_f64).to_string(),
            None => "".to_string(),
        },
        match more_meta.is_empty() {
            true => "".to_string(),
            false => format!("{}", more_meta.join(", ")).as_str().to_string(),
        },
        match is_temporary {
            true => format!(
                "any {} (temporary)",
                match affix_def.affix_location {
                    AffixLocationEnum::Prefix => "prefix",
                    AffixLocationEnum::Suffix => "suffix",
                    _ => "???",
                }
            )
            .to_string(),
            false => name.to_string(),
        }
    )
    .unwrap();
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::prelude::pymethods)]
impl ItemRoute {
    #[pyo3(name = "to_pretty_string")]
    pub fn to_pretty_string_py(
        &self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        statistic_analyzer: &DynStatisticAnalyzerPaths,
        calculator: &Calculator,
        groups: Option<Vec<GroupRoute>>,
    ) -> String {
        self.to_pretty_string(
            item_provider,
            market_provider,
            statistic_analyzer.0.as_ref(),
            calculator,
            groups.as_ref(),
        )
    }

    #[pyo3(name = "locate_group")]
    pub fn locate_group_py(&self, calculated_groups: Vec<GroupRoute>) -> Option<GroupRoute> {
        self.locate_group(calculated_groups.as_ref()).cloned()
    }
}
