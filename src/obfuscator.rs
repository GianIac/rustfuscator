/// Macro to obfuscate a string literal at compile-time.
#[macro_export]
macro_rules! obfuscate_string {
    ($s:literal) => {
        cryptify::encrypt_string!($s)
    };
}

/// Macro per obfuscation del control-flow.
#[macro_export]
macro_rules! obfuscate_flow {
    () => {
        cryptify::flow_stmt!()
    };
}
