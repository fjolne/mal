pub mod env;
pub mod printer;
pub mod reader;
use std::{collections::HashMap, io::Write, rc::Rc};

use env::Env;
use reader::*;

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

impl Invoke for MalFn {
    fn invoke(&self, args: Vec<MalForm>) -> Result<MalForm, MalErr> {
        let mut let_body = vec![MalForm::Symbol("let*".to_owned())];
        let mut bindings = vec![];
        for (i, p) in self.params.iter().enumerate() {
            bindings.push(MalForm::Symbol(p.clone()));
            bindings.push(args[i].clone());
        }
        let_body.push(MalForm::List(bindings));
        let_body.push((*self.body).clone());
        EVAL(MalForm::List(let_body), &mut self.env.clone())
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

    fn eval_default(form: MalForm, env: &mut Env) -> Result<MalForm, MalErr> {
        let MalForm::List(ref v) = eval_ast(&form, env)? else {
            panic!();
        };
        let [head, args @ ..] = &v[..] else {
            panic!();
        };
        match head {
            MalForm::FnSpecial(invokable) => invokable.invoke(args.to_vec()),
            MalForm::Fn(invokable) => invokable.invoke(args.to_vec()),
            _ => Err("head of a list is not invokable".to_owned()),
        }
    }
    fn wrap_do(body: &[MalForm]) -> MalForm {
        let mut do_body_vec = vec![MalForm::Symbol("do".to_owned())];
        do_body_vec.append(&mut body.to_owned());
        let do_body = MalForm::List(do_body_vec);
        do_body
    }
    match &v[..] {
        [] => Ok(form),
        [MalForm::Symbol(symbol), args @ ..] => match symbol.as_str() {
            "do" => {
                let MalForm::List (args) = eval_ast(&MalForm::List(args.to_owned()), env)? else {
                    panic!();
                };
                Ok(args.last().cloned().unwrap_or(MalForm::Nil))
            }
            "fn*" => {
                let [MalForm::List(params) | MalForm::Vector(params), body @ ..] = args else {
                    return Err("fn* form must have a seq as 2nd arg".to_owned());
                };
                let params = params
                    .into_iter()
                    .map(|x| {
                        if let MalForm::Symbol(s) = x {
                            Ok(s.to_owned())
                        } else {
                            Err("fn parameter must be a symbol".to_owned())
                        }
                    })
                    .collect::<Result<Vec<String>, MalErr>>()?;
                Ok(MalForm::Fn(MalFn {
                    env: Box::new(Env::with_outer(env)),
                    params,
                    body: Box::new(wrap_do(body)),
                }))
            }
            "if" => {
                let [cond_expr, true_statement, false_statements @ ..] = args else {
                    return Err("'if' should have at least 2 args".to_owned());
                };
                let MalForm::Bool(cond_expr) = EVAL(cond_expr.clone(), env)? else {
                    return Err("'if' cond_expr should evaluate to bool".to_owned());
                };
                EVAL(
                    if cond_expr {
                        true_statement.clone()
                    } else {
                        wrap_do(false_statements)
                    },
                    env,
                )
            }
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
                    return Err("let form must have a seq as 2nd arg".to_owned());
                };
                let mut let_env = Env::with_outer(env);
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
            _ => eval_default(form, env),
        },
        _ => eval_default(form, env),
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
    MalForm::FnSpecial(MalFnSpecial::new(Rc::new(
        move |args: Vec<MalForm>| -> MalForm {
            MalForm::Int(f(args
                .iter()
                .map(|x| {
                    let MalForm::Int(n) = x else {
                               panic!("not a number");
                           };
                    *n
                })
                .collect::<Vec<i64>>()))
        },
    )))
}

fn cmp(a: &MalForm, b: &MalForm) -> i64 {
    match (a, b) {
        (MalForm::Int(x), MalForm::Int(y)) => x - y,
        _ => -1,
    }
}

fn new_cmp_fn<G>(cmp_cond: G) -> MalForm
where
    G: Fn(i64) -> bool + 'static,
{
    MalForm::FnSpecial(MalFnSpecial::new(Rc::new(
        move |args: Vec<MalForm>| -> MalForm { MalForm::Bool(cmp_cond(cmp(&args[0], &args[1]))) },
    )))
}

fn default_env() -> Env {
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
        ("=".to_string(), new_cmp_fn(|x| x == 0)),
        ("<".to_string(), new_cmp_fn(|x| x < 0)),
        ("<=".to_string(), new_cmp_fn(|x| x <= 0)),
        (">".to_string(), new_cmp_fn(|x| x > 0)),
        (">=".to_string(), new_cmp_fn(|x| x >= 0)),
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
