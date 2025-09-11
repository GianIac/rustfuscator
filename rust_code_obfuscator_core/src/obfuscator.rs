#[macro_export]
macro_rules! obfuscate_string {
    ($s:literal) => {
        cryptify::encrypt_string!($s)
    };
    ($other:expr) => {
        compile_error!("obfuscate_string! only accepts string literals");
    };
}

#[macro_export]
macro_rules! obfuscate_flow {
    () => {
        cryptify::flow_stmt!()
    };
    ($($t:tt)*) => {
        compile_error!("obfuscate_flow! does not accept arguments");
    };
}
