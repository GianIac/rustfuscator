use rust_code_obfuscator::obfuscate_num;

#[test]
fn obfuscate_num_round_trips_from_public_crate() {
    let unsigned: u64 = obfuscate_num!(9_000_000_000u64);
    let signed: i32 = obfuscate_num!(-42i32);
    let inferred = obfuscate_num!(1234);
    let inferred_from_context: u64 = obfuscate_num!(1234);

    assert_eq!(unsigned, 9_000_000_000u64);
    assert_eq!(signed, -42i32);
    assert_eq!(inferred, 1234);
    assert_eq!(inferred_from_context, 1234u64);
}

#[test]
fn exported_macros_do_not_require_direct_cryptify_dependency() {
    assert_eq!(
        rust_code_obfuscator::obfuscate_string!("standalone consumer").as_str(),
        "standalone consumer"
    );
    rust_code_obfuscator::obfuscate_flow!();
}
