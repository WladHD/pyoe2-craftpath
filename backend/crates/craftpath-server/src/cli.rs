//! The classic interactive CLI, now a subcommand of the backend binary
//! (`pyoe2-backend cli`). Behavior is unchanged from the former
//! `pyoe2_craftpath_cli` binary.

use std::{io::Read, path::Path};

use craftpath_core::progress::ProgressSink;
use craftpath_core::utils::{
    logger_utils::init_tracing, ram_input_utils::parse_human_size,
    version_checker_utils::check_new_version,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Forwards core progress messages to a terminal spinner (the same UX the
/// collectors used to render directly before the ProgressSink hook).
struct IndicatifSink(ProgressBar);

impl IndicatifSink {
    fn new_spinner() -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠚⠞⠖⠦⠴⠲⠳⠓ "),
        );
        pb.enable_steady_tick(Duration::from_millis(500));
        Self(pb)
    }
}

impl ProgressSink for IndicatifSink {
    fn report(&self, message: &str, _current: u64, _total: Option<u64>) {
        self.0.set_message(message.to_string());
    }
}
use anyhow::{Result, anyhow};
use clap::{ArgAction, ValueHint};
use humansize::SizeFormatter;
use craftpath_core::{
    GITHUB_REPOSITORY,
    api::{
        calculator::{Calculator, GroupRoute},
        types::THashMap,
    },
    calc::{
        matrix::presets::matrix_builder_presets::MatrixBuilderPreset,
        statistics::presets::{
            statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset,
            statistic_analyzer_path_presets::StatisticAnalyzerPathPreset,
        },
    },
    external_api::{
        coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
        coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport,
        fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
        pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider,
    },
};

pub fn run_cli(args: CliArgs) {
    ctrlc::set_handler(|| std::process::exit(2)).unwrap();
    init_tracing();
    tracing::info!("Starting PyoE2 CraftPath CLI");

    let res = calc_pipleline(args);

    match res {
        Err(e) => tracing::error!("{:#?}", e),
        _ => {}
    }

    println!("Press Enter to close ...");
    let _ = std::io::stdin().read(&mut [0u8]).unwrap();
}

#[derive(clap::Args, Debug)]
pub struct CliArgs {
    /// Path to starting item file (in JSON format, exported from craftofexile.com's emulator).
    #[arg(
        short,
        long,
        env = "START_ITEM_PATH",
        default_value = "pyoe2-craftpath/startitem.json",
        value_hint = ValueHint::FilePath
    )]
    start_item_path: String,

    /// Path to target item file (in JSON format, exported from craftofexile.com's emulator).
    #[arg(
        short,
        long,
        env = "TARGET_ITEM_PATH",
        default_value = "pyoe2-craftpath/targetitem.json",
        value_hint = ValueHint::FilePath
    )]
    target_item_path: String,

    /// Path to cache directory to drop data from Craft of Exile and PoE Ninja.
    #[arg(
        short, long, env = "CACHE_PATH", 
        default_value = "pyoe2-craftpath", 
        value_hint = ValueHint::DirPath
    )]
    cache_path: String,

    /// Number of grouped routes and unique paths to search for and print to console.
    #[arg(short, long, env = "AMOUNT_ROUTES", default_value_t = 5)]
    amount_routes: u32,

    #[arg(
        short,
        long,
        env = "POE2_LEAGUE",
        default_value = "
        "
    )]
    poe2_league: String,

    /// Set flag if you do not want to check for updates at the start of the program.
    #[arg(long, env = "NO_UPDATES", action = ArgAction::SetTrue)]
    no_updates: bool,

    /// Do not calculate memory intensive group information.
    #[arg(long, env = "NO_GROUPS", action = ArgAction::SetTrue)]
    no_groups: bool,

    /// !!! USE AT YOUR OWN RISK.
    /// Maximal RAM this program is allowed to use (accepts "4 GB", "512 MB", "2G", etc.)
    /// Depending on your system, reaching maximal RAM might crash it ... with all the side-effects that come with it.
    /// So make sure to provide a sensible max_ram value.
    #[arg(long, env = "MAX_RAM", default_value = "1G", value_parser = parse_human_size)]
    max_ram: u64,
}

fn calc_pipleline(args: CliArgs) -> Result<()> {

    tracing::info!("You can run the program with '--help' to see all available options.");

    tracing::info!(
        "Running pipeline with config:\n\
        start_item_path: {}\n\
        target_item_path: {}\n\
        cache_path: {}\n\
        amount_routes: {}\n\
        poe2_league: {}\n\
        max_ram: {}\n\
        disable group calculation? {}\n\
        check for updates? {}",
        args.start_item_path,
        args.target_item_path,
        args.cache_path,
        args.amount_routes,
        args.poe2_league,
        SizeFormatter::new(args.max_ram, humansize::DECIMAL),
        match args.no_groups {
            true => "disable",
            false => "calculate",
        },
        match args.no_updates {
            true => "ain't nobody got time for that",
            false => "check",
        },
    );

    if !args.no_updates {
        let res = check_new_version(GITHUB_REPOSITORY);

        match res {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(
                    "Could not check for updates. Program will continue.\nError: {:?}",
                    e
                )
            }
        };
    }

    let res = start_calc_and_print(&args);

    match res {
        Ok(e) => Ok(e),
        Err(e) => {
            tracing::error!(
                "An error occurred. Run program with '--help' to see possible configuration options."
            );
            Err(e)
        }
    }
}

fn check_cache_path(cache_path: &str) -> Result<()> {
    let path = Path::new(cache_path);

    if !path.exists() {
        return Err(anyhow!(
            "Cache path '{}' does not exist. Explicitly create it.",
            cache_path
        ));
    }

    if !path.is_dir() {
        return Err(anyhow!("Cache path '{}' is not a directory", cache_path));
    }

    Ok(())
}

fn start_calc_and_print(args: &CliArgs) -> Result<()> {
    check_cache_path(args.cache_path.as_str())?;

    let item_provider_hm = THashMap::from_iter(vec![(
        format!("{}/coe2.json", args.cache_path).to_string(),
        "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
    )]);

    let economy_provider_hm = THashMap::from_iter(vec![
        (
            format!("{}/pn_abyss.json", args.cache_path).to_string(),
            format!("https://poe.ninja/poe2/api/economy/exchange/current/overview?league={}&type=Abyss", args.poe2_league.as_str()).to_string(),
        ),
        (
            format!("{}/pn_currency.json", args.cache_path).to_string(),
            format!("https://poe.ninja/poe2/api/economy/exchange/current/overview?league={}&type=Currency", args.poe2_league.as_str()).to_string(),
        ),
        (
            format!("{}/pn_essences.json", args.cache_path).to_string(),
            format!("https://poe.ninja/poe2/api/economy/exchange/current/overview?league={}&type=Essences", args.poe2_league.as_str()).to_string(),
        ),
        (
            format!("{}/pn_ritual.json", args.cache_path).to_string(),
            format!("https://poe.ninja/poe2/api/economy/exchange/current/overview?league={}&type=Ritual", args.poe2_league.as_str()).to_string(),
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

    let start_item = {
        tracing::info!("Reading contents from '{}' ...", args.start_item_path);
        let i1 = std::fs::read_to_string(args.start_item_path.as_str())?;
        CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(&i1, &item_provider)?
    };

    // for enditem only affixes relevant, sanity check equality before tho
    let target_item = {
        tracing::info!("Reading contents from '{}' ...", args.target_item_path);
        let i2 = std::fs::read_to_string(args.target_item_path.as_str())?;
        CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(&i2, &item_provider)?
    };

    let sink = IndicatifSink::new_spinner();

    let calculator = Calculator::generate_item_matrix_with_progress(
        start_item,
        target_item,
        &item_provider,
        &market_info,
        MatrixBuilderPreset::HappyPathMatrixBuilder
            .get_instance()
            .0
            .as_ref(),
        &sink,
    )?;

    let chance_inst = StatisticAnalyzerPathPreset::UniquePathChance.get_instance();

    let best_routes_chance = calculator.calculate_statistics_with_progress(
        &item_provider,
        &market_info,
        args.amount_routes,
        args.max_ram,
        chance_inst.0.as_ref(),
        &sink,
    )?;

    let cost_inst = StatisticAnalyzerPathPreset::UniquePathCost.get_instance();

    let best_routes_cost = calculator.calculate_statistics_with_progress(
        &item_provider,
        &market_info,
        args.amount_routes,
        args.max_ram,
        cost_inst.0.as_ref(),
        &sink,
    )?;

    let efficient_cost_inst = StatisticAnalyzerPathPreset::UniquePathEfficiency.get_instance();

    let best_routes_efficient_cost = calculator.calculate_statistics_with_progress(
        &item_provider,
        &market_info,
        args.amount_routes,
        args.max_ram,
        efficient_cost_inst.0.as_ref(),
        &sink,
    )?;

    let mut groups: Option<Vec<GroupRoute>> = None;

    if !args.no_groups {
        let group_instance =
            StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance.get_instance();

        groups = Some(
            calculator.calculate_statistics_currency_group_with_progress(
                &item_provider,
                &market_info,
                args.max_ram,
                StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance
                    .get_instance()
                    .0
                    .as_ref(),
                &sink,
            )?,
        );

        for group in groups
            .as_ref()
            .unwrap()
            .iter()
            .take(args.amount_routes as usize)
        {
            let out =
                group.to_pretty_string(&item_provider, &market_info, group_instance.0.as_ref());

            tracing::info!("{}", out);
        }
    }

    sink.0.finish_and_clear();

    for (analyzer, routes) in vec![
        (&chance_inst, best_routes_chance),
        (&efficient_cost_inst, best_routes_efficient_cost),
        (&cost_inst, best_routes_cost),
    ] {
        tracing::warn!("Printing results for '{}'", analyzer.0.get_name());

        for br in routes {
            let out = br.to_pretty_string(
                &item_provider,
                &market_info,
                cost_inst.0.as_ref(),
                &calculator,
                groups.as_ref(),
            );
            println!("{}", out);
        }
    }

    Ok(())
}
