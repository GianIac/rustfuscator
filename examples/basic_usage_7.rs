fn main() {
    // This is a simple example of obfuscating a string using cli: cargo run -p obfuscator_cli -- -i examples/basic_usage_7.rs -o output/
    let secret = "Codice segreto!";
    println!("--> {}", secret);

    let message = "Messaggio importante!";
    println!("Messaggio: {}", message);
}
