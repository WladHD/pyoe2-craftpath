fn main() {
    #[cfg(not(feature = "python"))]
    run_cli();
}

#[cfg(not(feature = "python"))]
pub fn init_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt().with_target(false).try_init();
    });
}

#[cfg(not(feature = "python"))]
fn run_cli() {
    init_tracing();
    tracing::info!("Starting PyoE2 CraftPath CLI");
}
