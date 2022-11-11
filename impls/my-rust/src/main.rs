pub mod env;
pub mod printer;
pub mod reader;
use std::{collections::HashMap, io::Write, rc::Rc};

use env::Env;
use reader::{MalErr, MalForm};

fn eval_ast(form: &MalForm, env: &mut Env) -> Result<MalForm, MalErr> {
    let mut eval_seq = |v: &Vec<MalForm>| -> Result<Vec<MalForm>, MalErr> {
        v.iter()
            .map(|form| EVAL(form.clone(), env))
            .collect::<Result<Vec<MalForm>, MalErr>>()
    };
    match form {
        MalForm::Symbol(s) => env.get(s).cloned().ok_or(format!("symbol '{s}' not found")),
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

fn rep(input: String, env: &mut Env) -> Result<String, MalErr> {
    PRINT(EVAL(READ(input)?, env)?)
}

#[allow(non_snake_case)]
fn READ(input: String) -> Result<MalForm, MalErr> {
    reader::read_str(input)
}

#[allow(non_snake_case)]
fn EVAL(form: MalForm, env: &mut Env) -> Result<MalForm, MalErr> {
    let MalForm::List(ref v) = form else {
        return eval_ast(&form, env);
    };
    match &v[..] {
        [] => Ok(form),
        [MalForm::Symbol(symbol), args @ ..] => match symbol.as_str() {
            "def!" => {
                let [name, value] = args else {
                    return Err("wrong number of args to def!".to_owned());
                };
                let MalForm::Symbol(name) = name else {
                    return Err("not a symbol".to_owned());
                };
                let value = EVAL(value.clone(), env)?;
                Ok(env.set(name, value).clone())
            }
            "let*" => {
                let [MalForm::List(bindings) | MalForm::Vector(bindings), body @ ..] = args else {
                    return Err("let form must have a list as a 2nd arg".to_owned());
                };
                let mut let_env = Env::new();
                let_env.outer = Some(env);
                for kv in bindings.chunks(2) {
                    let [k, v] = kv else {
                        panic!();
                    };
                    let MalForm::Symbol(k) = k else {
                        return Err("odd left bindings should be symbols".to_owned());
                    };
                    let v = EVAL(v.clone(), &mut let_env)?;
                    let_env.set(k, v);
                }
                let body_list = MalForm::List(body.to_owned());
                let MalForm::List(xs) = eval_ast(&body_list, &mut let_env)? else {
                    panic!();
                };
                Ok(xs.last().cloned().unwrap_or(MalForm::Nil))
            }
            _ => {
                let MalForm::List(ref v) = eval_ast(&form, env)? else {
                    panic!();
                };
                let [MalForm::Function(f_ptr), args @ ..] = &v[..] else {
                    return Err("head of a list is neither a function, nor a special form".to_string());
                };
                Ok((*f_ptr)(args.to_vec()))
            }
        },
        _ => Err("head of a list is not a symbol".to_owned()),
    }
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

fn default_env() -> Env<'static> {
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
    .into()
}

fn main() -> Result<(), MalErr> {
    let mut repl_env = default_env();

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
        match rep(input, &mut repl_env) {
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
        let mut repl_env = default_env();
        assert_eq!(
            EVAL(READ("(+ 1 (* 2 3) (/ 7 3) (- 4 1))".into())?, &mut repl_env)?,
            MalForm::Int(12)
        );
        Ok(())
    }
}
