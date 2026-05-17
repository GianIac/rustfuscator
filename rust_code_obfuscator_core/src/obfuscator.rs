#[macro_export]
macro_rules! obfuscate_string {
    ($s:literal) => {{
        fn init() -> &'static str {
            static CELL: ::std::sync::OnceLock<&'static str> = ::std::sync::OnceLock::new();
            *CELL.get_or_init(|| {
                let decrypted: ::std::string::String = cryptify::encrypt_string!($s);
                $crate::obfuscator::__verify_literal_round_trip($s, decrypted.as_str());
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
macro_rules! obfuscate_num {
    (-$n:literal) => {{
        let rustfuscator_value = -$n;
        $crate::obfuscator::__obfuscate_num_value(
            rustfuscator_value,
            (::core::line!() as u64)
                ^ ((::core::column!() as u64) << 32)
                ^ 0xa24b_aed4_963e_e407u64,
        )
    }};
    ($n:literal) => {{
        let rustfuscator_value = $n;
        $crate::obfuscator::__obfuscate_num_value(
            rustfuscator_value,
            (::core::line!() as u64)
                ^ ((::core::column!() as u64) << 32)
                ^ 0xa24b_aed4_963e_e407u64,
        )
    }};
    ($($t:tt)*) => {
        compile_error!("obfuscate_num! only accepts integer literals");
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

#[doc(hidden)]
#[inline]
pub fn __verify_literal_round_trip(original: &str, decrypted: &str) {
    #[cfg(all(debug_assertions, feature = "verify_literals"))]
    {
        debug_assert_eq!(
            decrypted, original,
            "rustfuscator literal verification failed: decrypted value differs from original"
        );
    }

    #[cfg(not(all(debug_assertions, feature = "verify_literals")))]
    {
        let _ = (original, decrypted);
    }
}

#[doc(hidden)]
pub trait __ObfuscateNum: Copy {
    fn __obfuscate_num(self, seed: u64) -> Self;
}

macro_rules! impl_obfuscate_num {
    ($($ty:ty),* $(,)?) => {
        $(
            impl __ObfuscateNum for $ty {
                #[inline]
                fn __obfuscate_num(self, seed: u64) -> Self {
                    let mask = ::core::hint::black_box(seed as $ty);
                    let offset = ::core::hint::black_box(mask.rotate_left(7));
                    let encoded = (self ^ mask).wrapping_add(offset);
                    ::core::hint::black_box(encoded.wrapping_sub(offset) ^ mask)
                }
            }
        )*
    };
}

impl_obfuscate_num!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[doc(hidden)]
#[inline]
pub fn __obfuscate_num_value<T: __ObfuscateNum>(value: T, seed: u64) -> T {
    value.__obfuscate_num(seed)
}

#[cfg(test)]
mod tests {
    #[test]
    fn obfuscate_string_macro_round_trips_literal() {
        assert_eq!(
            crate::obfuscate_string!("verified literal").as_str(),
            "verified literal"
        );
    }

    #[test]
    fn obfuscate_num_round_trips_unsigned_integer_literals() {
        assert_eq!(crate::obfuscate_num!(0u8), 0u8);
        assert_eq!(crate::obfuscate_num!(255u8), 255u8);
        assert_eq!(crate::obfuscate_num!(65_535u16), 65_535u16);
        assert_eq!(crate::obfuscate_num!(4_294_967_295u32), u32::MAX);
        assert_eq!(
            crate::obfuscate_num!(18_446_744_073_709_551_615u64),
            u64::MAX
        );
        assert_eq!(
            crate::obfuscate_num!(340282366920938463463374607431768211455u128),
            u128::MAX
        );
        assert_eq!(crate::obfuscate_num!(123usize), 123usize);
    }

    #[test]
    fn obfuscate_num_round_trips_signed_integer_literals() {
        assert_eq!(crate::obfuscate_num!(1234), 1234);
        assert_eq!(crate::obfuscate_num!(-42i8), -42i8);
        assert_eq!(crate::obfuscate_num!(-123i16), -123i16);
        assert_eq!(crate::obfuscate_num!(-123_456i32), -123_456i32);
        assert_eq!(crate::obfuscate_num!(-123_456_789i64), -123_456_789i64);
        assert_eq!(
            crate::obfuscate_num!(-123_456_789_123_456_789i128),
            -123_456_789_123_456_789i128
        );
        assert_eq!(crate::obfuscate_num!(-123isize), -123isize);
    }

    #[test]
    fn dummy_branch_macro_is_safe_to_execute() {
        crate::obfuscate_dummy_branch!();
    }

    #[cfg(all(debug_assertions, feature = "verify_literals"))]
    #[test]
    #[should_panic(expected = "rustfuscator literal verification failed")]
    fn literal_verification_panics_on_mismatch_when_enabled() {
        super::__verify_literal_round_trip("original", "different");
    }
}
