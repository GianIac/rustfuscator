#[macro_export]
macro_rules! obfuscate_string {
    ($s:literal) => {
        cryptify::encrypt_string!($s)
    };
}

#[macro_export]
macro_rules! obfuscate_flow {
    () => {
        cryptify::flow_stmt!()
    };
}
