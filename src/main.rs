pub mod api;
pub mod external_api;
pub mod utils;

fn main() {
    #[cfg(not(feature = "python"))]
    run_cli();
}

#[cfg(not(feature = "python"))]
fn run_cli() {
    use crate::utils::logger_utils::init_tracing;

    init_tracing();
    tracing::info!("Starting PyoE2 CraftPath CLI");
}
