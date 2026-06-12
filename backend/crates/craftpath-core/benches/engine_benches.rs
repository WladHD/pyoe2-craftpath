//! End-to-end criterion benches for the calculation engine.
//!
//! Fully offline: provider JSON is read straight from the bench cache
//! (default `backend/python_examples/cache/`, override with
//! `PYOE2_BENCH_CACHE`) - seed it once with
//! `backend/scripts/bench/warm_cache.sh`. The benched item pairs live in
//! `backend/benches/cases.json`, shared with the baseline-vs-head harness
//! `backend/scripts/bench/compare_engines.sh`.

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use craftpath_core::features::data::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;
use craftpath_core::features::data::coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport;
use craftpath_core::features::data::poe_ninja::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider;
use craftpath_core::prelude::*;

/// Mirror the CLI defaults so criterion numbers stay comparable to the
/// `compare_engines.sh` wall-clock numbers.
const AMOUNT_ROUTES: u32 = 5;
const MAX_RAM_BYTES: u64 = 1_000_000_000;

#[derive(serde::Deserialize)]
struct BenchCase {
    name: String,
    start: String,
    target: String,
}

/// `backend/` (the cargo workspace root), resolved from this crate's dir.
fn backend_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve backend workspace root")
}

fn cache_dir() -> PathBuf {
    env::var_os("PYOE2_BENCH_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| backend_root().join("python_examples/cache"))
}

fn read_cache_file(name: &str) -> String {
    let path = cache_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "bench cache file '{}' is missing ({err}); seed it with backend/scripts/bench/warm_cache.sh",
            path.display()
        )
    })
}

fn load_providers() -> (ItemInfoProvider, MarketPriceProvider) {
    let item_provider = CraftOfExileItemInfoProvider::parse_from_json(&read_cache_file("coe2.json"))
        .expect("parse coe2.json");
    let economy_jsons: Vec<String> = [
        "pn_abyss.json",
        "pn_currency.json",
        "pn_essences.json",
        "pn_ritual.json",
    ]
    .into_iter()
    .map(read_cache_file)
    .collect();
    let market_info = PoeNinjaMarketPriceProvider::parse_from_json_list(&economy_jsons)
        .expect("parse pn_*.json");
    (item_provider, market_info)
}

fn load_cases(item_provider: &ItemInfoProvider) -> Vec<(String, ItemSnapshot, ItemSnapshot)> {
    let manifest = backend_root().join("benches/cases.json");
    let raw = fs::read_to_string(&manifest)
        .unwrap_or_else(|err| panic!("read '{}': {err}", manifest.display()));
    let cases: Vec<BenchCase> = serde_json::from_str(&raw).expect("parse benches/cases.json");

    cases
        .into_iter()
        .map(|case| {
            let parse_item = |rel: &str| {
                let path = backend_root().join(rel);
                let json = fs::read_to_string(&path)
                    .unwrap_or_else(|err| panic!("read item '{}': {err}", path.display()));
                CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(
                    &json,
                    item_provider,
                )
                .unwrap_or_else(|err| panic!("parse item '{}': {err:?}", path.display()))
            };
            (case.name, parse_item(&case.start), parse_item(&case.target))
        })
        .collect()
}

fn bench_matrix_generation(c: &mut Criterion) {
    let (item_provider, market_info) = load_providers();
    let cases = load_cases(&item_provider);
    let builder = MatrixBuilderPreset::HappyPathMatrixBuilder.get_instance();

    let mut group = c.benchmark_group("matrix_generation");
    for (name, start, target) in &cases {
        group.bench_function(name.as_str(), |b| {
            b.iter_batched(
                || (start.clone(), target.clone()),
                |(start, target)| {
                    Calculator::generate_item_matrix(
                        start,
                        target,
                        &item_provider,
                        &market_info,
                        builder.0.as_ref(),
                    )
                    .expect("matrix generation")
                },
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

fn bench_statistics_chance(c: &mut Criterion) {
    let (item_provider, market_info) = load_providers();
    let cases = load_cases(&item_provider);
    let builder = MatrixBuilderPreset::HappyPathMatrixBuilder.get_instance();
    let analyzer = StatisticAnalyzerPathPreset::UniquePathChance.get_instance();

    let mut group = c.benchmark_group("statistics_unique_path_chance");
    for (name, start, target) in cases {
        let calculator = Calculator::generate_item_matrix(
            start,
            target,
            &item_provider,
            &market_info,
            builder.0.as_ref(),
        )
        .expect("matrix generation");
        group.bench_function(name.as_str(), |b| {
            b.iter(|| {
                calculator
                    .calculate_statistics(
                        &item_provider,
                        &market_info,
                        AMOUNT_ROUTES,
                        MAX_RAM_BYTES,
                        analyzer.0.as_ref(),
                    )
                    .expect("statistics")
            })
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    // A single end-to-end iteration can take seconds, so stay at criterion's
    // minimum sample size with a generous measurement window.
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
    targets = bench_matrix_generation, bench_statistics_chance
}
criterion_main!(benches);
