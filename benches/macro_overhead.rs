use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_code_obfuscator::{obfuscate_flow, obfuscate_num, obfuscate_string};

fn baseline_math(input: u64) -> u64 {
    input.wrapping_mul(1_315_423_911).rotate_left(7) ^ 0xa5a5_a5a5_a5a5_a5a5
}

fn obfuscated_flow_math(input: u64) -> u64 {
    obfuscate_flow!();
    input.wrapping_mul(1_315_423_911).rotate_left(7) ^ 0xa5a5_a5a5_a5a5_a5a5
}

fn baseline_string_len() -> usize {
    "rustfuscator benchmark literal".len()
}

fn obfuscated_string_len() -> usize {
    obfuscate_string!("rustfuscator benchmark literal").len()
}

fn baseline_num() -> u64 {
    9_000_000_000u64
}

fn obfuscated_num() -> u64 {
    obfuscate_num!(9_000_000_000u64)
}

fn bench_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("control_flow");
    group.bench_function("baseline_math", |b| {
        b.iter(|| baseline_math(black_box(42)));
    });
    group.bench_function("obfuscated_flow_math", |b| {
        b.iter(|| obfuscated_flow_math(black_box(42)));
    });
    group.finish();
}

fn bench_literals(c: &mut Criterion) {
    let mut group = c.benchmark_group("literals");
    group.bench_function("baseline_string_len", |b| {
        b.iter(|| black_box(baseline_string_len()));
    });
    group.bench_function("obfuscated_string_len", |b| {
        b.iter(|| black_box(obfuscated_string_len()));
    });
    group.bench_function("baseline_num", |b| {
        b.iter(|| black_box(baseline_num()));
    });
    group.bench_function("obfuscated_num", |b| {
        b.iter(|| black_box(obfuscated_num()));
    });
    group.finish();
}

criterion_group!(benches, bench_flow, bench_literals);
criterion_main!(benches);
