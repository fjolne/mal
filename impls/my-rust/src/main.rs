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
        MalForm::Map(m) => Ok(MalForm::Map(
            m.iter()
                .map(|(k, v)| Ok((k.clone(), EVAL(v.clone(), env)?)))
                .collect::<Result<HashMap<String, MalForm>, MalErr>>()?,
        )),
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
    let form = eval_ast(&form, env)?;
    let MalForm::List(ref v) = form else {
        return Ok(form);
    };
    if v.is_empty() {
        return Ok(form);
    }
    let [MalForm::Function(f_ptr), args @ ..] = &v[..] else {
        return Err("head of a list is not a function".to_string());
    };
    Ok((*f_ptr)(args.to_vec()))
}

#[allow(non_snake_case)]
fn PRINT(input: MalForm) -> Result<String, MalErr> {
    Ok(printer::pr_str(&input))
}

fn new_arithmetic_fn<F>(f: F) -> MalForm
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

fn new_repl_env() -> ReplEnv {
    HashMap::from([
        (
            "+".to_string(),
            new_arithmetic_fn(|xs| xs.into_iter().fold(0, |a, b| a + b)),
        ),
        (
            "-".to_string(),
            new_arithmetic_fn(|xs| xs.into_iter().reduce(|a, b| a - b).unwrap()),
        ),
        (
            "*".to_string(),
            new_arithmetic_fn(|xs| xs.into_iter().fold(1, |a, b| a * b)),
        ),
        (
            "/".to_string(),
            new_arithmetic_fn(|xs| xs.into_iter().reduce(|a, b| a / b).unwrap()),
        ),
    ])
}

fn main() -> Result<(), MalErr> {
    let repl_env = new_repl_env();

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
        let repl_env = new_repl_env();
        assert_eq!(
            EVAL(READ("(+ 1 (* 2 3) (/ 7 3) (- 4 1))".into())?, &repl_env)?,
            MalForm::Int(12)
        );
        Ok(())
    }
}
