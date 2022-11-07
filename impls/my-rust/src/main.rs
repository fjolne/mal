use std::io::Write;

fn rep(input: &str) -> &str {
    PRINT(EVAL(READ(input)))
}

fn PRINT(input: &str) -> &str {
    input
}

fn EVAL(input: &str) -> &str {
    input
}

fn READ(input: &str) -> &str {
    input
}

fn main() {
    loop {
        print!("user> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        let Ok(bytes_read) = std::io::stdin().read_line(&mut input) else {
            println!("Failed to read from stdin");
            break;
        };
        if bytes_read == 0 {
            println!("Goodbye!");
            break;
        }
        input.pop();
        println!("{}", rep(&input));
    }
}
