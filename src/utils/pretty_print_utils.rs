use num_format::{Locale, ToFormattedString};

use crate::api::{
    calculator::{DynStatisticAnalyzerCurrencyGroups, GroupRoute, StatisticAnalyzerCurrencyGroups},
    provider::{
        item_info::ItemInfoProvider,
        market_prices::{MarketPriceProvider, PriceInDivines, PriceKind},
    },
};
use std::fmt::Write;

impl GroupRoute {
    pub fn to_pretty_string(
        &self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        grouped_statistic_analyzer: &dyn StatisticAnalyzerCurrencyGroups,
    ) -> String {
        let mut out = String::new();

        let cost_per_1 = grouped_statistic_analyzer.calculate_cost_per_craft(
            &self.group,
            &item_provider,
            &market_provider,
        );

        let tries_for_60 = ((((1.0_f64 - 0.6_f64).ln()
            / (1.0_f64 - self.chance.get_raw_value()).ln())
        .ceil()) as u64)
            .max(1);

        let cost_per_60 =
            PriceInDivines::new((tries_for_60 as f64) * cost_per_1.get_divine_value());

        writeln!(
            &mut out,
            "Group Chance: {:.5}% | Unique Routes: {} | Tries needed for 60%: {} | Cost per Craft: {} | Cost for 60%: {}{}",
            self.chance.get_raw_value() * 100_f64,
            self.amount_subpaths.get_raw_value().to_formatted_string(&Locale::en),
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
            match grouped_statistic_analyzer.format_display_more_info(
                &self,
                &item_provider,
                &market_provider
            ) {
                Some(e) => e,
                None => "".to_string(),
            }
        )
        .unwrap();

        for (index, currency_list) in self.group.iter().enumerate() {
            let index_weight = grouped_statistic_analyzer.calculate_chance_for_group_step_index(
                &self.unique_route_weights,
                self.amount_subpaths.clone(),
                index,
            );

            writeln!(
                &mut out,
                "{}. {} [ROUGH (!) avg. chance: {:.5}%]",
                index + 1,
                currency_list
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
                index_weight.get_raw_value() * 100_f64
            )
            .unwrap();
        }

        out
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::prelude::pymethods)]
impl GroupRoute {
    #[pyo3(name = "to_pretty_string")]
    pub fn to_pretty_string_py(
        &self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        statistic_analyzer: &DynStatisticAnalyzerCurrencyGroups,
    ) -> String {
        self.to_pretty_string(
            item_provider,
            market_provider,
            statistic_analyzer.0.as_ref(),
        )
    }
}
