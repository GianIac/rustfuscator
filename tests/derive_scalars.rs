use rust_code_obfuscator::Obfuscate;

#[derive(Debug, PartialEq, Obfuscate)]
struct ScalarFixture {
    name: String,
    enabled: bool,
    small: u8,
    medium: i32,
    large: u64,
    negative: i64,
}

#[test]
fn derive_obfuscate_round_trips_supported_scalar_fields() {
    let obfuscated =
        ObfuscatedScalarFixture::new_clear("alice", true, 7, -42, 9_000_000_000, -9_000);

    let clear = obfuscated.get_clear();

    assert_eq!(
        clear,
        ScalarFixture {
            name: "alice".to_string(),
            enabled: true,
            small: 7,
            medium: -42,
            large: 9_000_000_000,
            negative: -9_000,
        }
    );
}
