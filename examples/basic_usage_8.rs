// This is a simple example of obfuscating a string and flow using cli: cargo run -p obfuscator_cli -- -i examples/basic_usage_8.rs -o output/
fn main() {
    // If / Else
    if true {
        println!("TRUE");
    } else {
        println!("FALSE");
    }

    // Match
    let numero = 2;
    match numero {
        1 => println!("Uno"),
        2 => println!("Due"),
        _ => println!("Altro"),
    }

    // Loop
    let mut counter = 0;
    loop {
        println!("Dentro loop infinito");
        counter += 1;
        if counter >= 1 {
            break;
        }
    }

    // While
    let mut x = 0;
    while x < 1 {
        println!("While x < 1");
        x += 1;
    }

    // For
    for i in 0..2 {
        println!("For loop: {}", i);
    }

    // String
    let messaggio = "OFFUSCAMI";
    println!("{}", messaggio);
}
