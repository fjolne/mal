pub mod printer;
pub mod reader;
use std::{collections::HashMap, io::Write, rc::Rc};

use reader::{MalErr, MalForm};

type ReplEnv = HashMap<String, MalForm>;

fn eval_ast(form: &MalForm, env: &ReplEnv) -> Result<MalForm, MalErr> {
    let eval_seq = |v: &Vec<MalForm>| -> Result<Vec<MalForm>, MalErr> {
        v.iter()
            .map(|form| EVAL(form.clone(), env))
            .collect::<Result<Vec<MalForm>, MalErr>>()
    };
    match form {
        MalForm::Symbol(s) => env.get(s).cloned().ok_or(format!("symbol not found: {s}")),
        MalForm::List(v) => Ok(MalForm::List(eval_seq(v)?)),
        MalForm::Vector(v) => Ok(MalForm::Vector(eval_seq(v)?)),
        _ => Ok(form.clone()),
    }
}

fn rep(input: String, env: &ReplEnv) -> Result<String, MalErr> {
    PRINT(EVAL(READ(input)?, env)?)
}

#[allow(non_snake_case)]
fn READ(input: String) -> Result<MalForm, MalErr> {
    reader::read_str(input)
}

#[allow(non_snake_case)]
fn EVAL(form: MalForm, env: &ReplEnv) -> Result<MalForm, MalErr> {
    match form {
        MalForm::List(ref v) => {
            if v.is_empty() {
                Ok(form)
            } else {
                let MalForm::List(v) = eval_ast(&form, env)? else {
                    panic!("evaluated list is not a list");
                };
                let MalForm::Function(f) = v.first().expect("list is empty") else {
                    panic!("first arg is not a function");
                };
                let args = &v[1..];
                Ok((*f)(args.to_vec()))
            }
        }
        _ => eval_ast(&form, env),
    }
}

#[allow(non_snake_case)]
fn PRINT(input: MalForm) -> Result<String, MalErr> {
    Ok(printer::pr_str(&input))
}

fn build_arithmetic_fn<F>(f: F) -> MalForm
where
    F: Fn(Vec<i64>) -> i64 + 'static,
{
    MalForm::Function(Rc::new(move |args: Vec<MalForm>| -> MalForm {
        MalForm::Int(f(args
            .iter()
            .map(|x| {
                let MalForm::Int(n) = x else {
                    panic!("not a number");
                };
                *n
            })
            .collect::<Vec<i64>>()))
    }))
}

fn build_repl_env() -> ReplEnv {
    HashMap::from([
        (
            "+".to_string(),
            build_arithmetic_fn(|xs| xs.into_iter().fold(0, |a, b| a + b)),
        ),
        (
            "-".to_string(),
            build_arithmetic_fn(|xs| xs.into_iter().reduce(|a, b| a - b).unwrap()),
        ),
        (
            "*".to_string(),
            build_arithmetic_fn(|xs| xs.into_iter().fold(1, |a, b| a * b)),
        ),
        (
            "/".to_string(),
            build_arithmetic_fn(|xs| xs.into_iter().reduce(|a, b| a / b).unwrap()),
        ),
    ])
}

fn main() -> Result<(), MalErr> {
    let repl_env = build_repl_env();

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
        match rep(input, &repl_env) {
            Ok(result) => println!("{}", result),
            Err(err) => println!("error: {}", err),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_test() -> Result<(), MalErr> {
        let repl_env = build_repl_env();
        assert_eq!(
            EVAL(READ("(+ 1 (* 2 3) (/ 7 3) (- 4 1))".into())?, &repl_env)?,
            MalForm::Int(12)
        );
        Ok(())
    }
}
