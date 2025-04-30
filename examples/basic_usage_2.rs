use rust_code_obfuscator::obfuscate_flow;

fn compute_something(mut num: u32) -> u32 {
    obfuscate_flow!();

    num = num.wrapping_mul(12345);
    num ^= 0xABCDEF12;
    num = num.rotate_left(7);
    num
}

fn main() {
    let result = compute_something(42);
    println!("Result: {}", result);
}