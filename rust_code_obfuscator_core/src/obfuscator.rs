#[macro_export]
macro_rules! obfuscate_string {
    ($s:literal) => {{
        fn init() -> &'static str {
            static CELL: ::std::sync::OnceLock<&'static str> = ::std::sync::OnceLock::new();
            *CELL.get_or_init(|| {
                let decrypted: ::std::string::String = cryptify::encrypt_string!($s);
                ::std::boxed::Box::leak(decrypted.into_boxed_str())
            })
        }
        $crate::ObfStr::new(init)
    }};
    ($other:expr) => {
        compile_error!("obfuscate_string! only accepts string literals");
    };
}

#[macro_export]
macro_rules! obfuscate_str {
    ($s:literal) => {{
        $crate::obfuscate_string!($s).as_str()
    }};
    ($other:expr) => {
        compile_error!("obfuscate_str! only accepts string literals");
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

#[macro_export]
macro_rules! obfuscate_dummy_branch {
    () => {{
        let rustfuscator_seed =
            ::core::line!() as u64 ^ ((::core::column!() as u64) << 32) ^ 0x9e37_79b9_7f4a_7c15u64;
        if ::core::hint::black_box(rustfuscator_seed.rotate_left(13)) == 0 {
            ::core::hint::black_box(rustfuscator_seed);
        }
    }};
    ($($t:tt)*) => {
        compile_error!("obfuscate_dummy_branch! does not accept arguments");
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_branch_macro_is_safe_to_execute() {
        crate::obfuscate_dummy_branch!();
    }
}
