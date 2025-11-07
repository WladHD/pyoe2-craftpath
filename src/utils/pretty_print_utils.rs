use crate::api::{
    calculator::{DynStatisticAnalyzerCurrencyGroups, GroupRoute, StatisticAnalyzerCurrencyGroups},
    provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
};
use std::fmt::Write;

impl GroupRoute {
    pub fn to_pretty_string(
        &self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        statistic_analyzer: &dyn StatisticAnalyzerCurrencyGroups,
    ) -> String {
        let mut out = String::new();

        let weight_for_60 = statistic_analyzer.calculate_weight_for_60_percent(
            &self,
            &item_provider,
            &market_provider,
        );

        writeln!(
            &mut out,
            "{}: {} - {} 60% Chance: {}{}",
            statistic_analyzer.template_group_weight_name(),
            statistic_analyzer.format_group_weight(self.weight),
            statistic_analyzer.template_60_percent_group_name(),
            statistic_analyzer.format_60_percent_group_weight(weight_for_60),
            match statistic_analyzer.format_display_more_info(
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
            let index_weight = statistic_analyzer
                .calculate_weight_for_group_step_index(&self.unique_route_weights, index);
            let index_weight_formatted =
                statistic_analyzer.template_weight_for_group_step_index(index_weight);

            writeln!(
                &mut out,
                "{}. \t{} [{}]",
                index + 1,
                currency_list
                    .list
                    .iter()
                    .map(|e| format!("{}", e.get_item_name(&item_provider)))
                    .collect::<Vec<String>>()
                    .join(" + "),
                index_weight_formatted
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
