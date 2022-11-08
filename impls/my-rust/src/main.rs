pub mod printer;
pub mod reader;

use std::io::Write;

use reader::{MalErr, MalForm};

fn rep(input: String) -> String {
    match READ(input) {
        Ok(form) => match EVAL(form) {
            Ok(result) => PRINT(result),
            Err(err) => err,
        },
        Err(err) => err,
    }
}

fn READ(input: String) -> Result<MalForm, MalErr> {
    reader::read_str(input)
}

fn EVAL(input: MalForm) -> Result<MalForm, MalErr> {
    Ok(input)
}

fn PRINT(input: MalForm) -> String {
    printer::pr_str(&input)
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
        println!("{}", rep(input));
    }
}
