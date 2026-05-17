use rust_code_obfuscator::Obfuscate;

#[derive(Debug, PartialEq, Obfuscate)]
struct ScalarFixture {
    name: String,
    enabled: bool,
    small: u8,
    medium: i32,
    large: u64,
    massive: u128,
    pointer_sized: usize,
    negative: i64,
    very_negative: i128,
    signed_pointer_sized: isize,
}

#[derive(Debug, PartialEq, Obfuscate)]
struct QualifiedStringFixture {
    standard: std::string::String,
}

#[test]
fn derive_obfuscate_round_trips_supported_scalar_fields() {
    let obfuscated = ObfuscatedScalarFixture::new_clear(
        "alice",
        true,
        7,
        -42,
        9_000_000_000,
        u128::MAX,
        usize::MAX,
        -9_000,
        i128::MIN,
        isize::MIN,
    );

    let clear = obfuscated.get_clear();

    assert_eq!(
        clear,
        ScalarFixture {
            name: "alice".to_string(),
            enabled: true,
            small: 7,
            medium: -42,
            large: 9_000_000_000,
            massive: u128::MAX,
            pointer_sized: usize::MAX,
            negative: -9_000,
            very_negative: i128::MIN,
            signed_pointer_sized: isize::MIN,
        }
    );
}

#[test]
fn derive_obfuscate_accepts_qualified_string_fields() {
    let obfuscated = ObfuscatedQualifiedStringFixture::new_clear("qualified");

    assert_eq!(
        obfuscated.get_clear(),
        QualifiedStringFixture {
            standard: "qualified".to_string(),
        }
    );
}
