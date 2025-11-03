#[macro_export]
macro_rules! derive_DebugDisplay {
    ($($t:ident),+ $(,)?) => {
        $(
            impl std::fmt::Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match serde_json::to_string_pretty(self) {
                        Ok(json) => write!(f, "{}", json),
                        Err(e) => write!(f, "<failed to serialize {}: {}>\nPrinting raw Rust debug string: {:#?}", stringify!($t), e, self),
                    }
                }
            }
        )+
    };
}
