#[cfg(not(feature = "python"))]
#[allow(dead_code)] // somehow this gets marked as unused when running in cli mode .. although its called first... idk
pub fn init_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt().with_target(false).try_init();
    });
}
