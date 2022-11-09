use std::rc::Rc;

use regex::Regex;

struct Reader {
    tokens: Vec<String>,
    cursor: usize,
}

impl Reader {
    fn next(&mut self) -> &str {
        let token = &self.tokens[self.cursor];
        self.cursor += 1;
        token
    }
    fn peek(&self) -> &str {
        &self.tokens[self.cursor]
    }
    fn eof(&self) -> bool {
        self.cursor >= self.tokens.len()
    }
}

fn tokenize(text: String) -> Vec<String> {
    let re = Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)
        .unwrap();
    re.captures_iter(&text)
        .map(|cap| String::from(cap.get(1).unwrap().as_str()))
        .filter(|token| !token.is_empty())
        .collect::<Vec<String>>()
}

pub type MalErr = String;

pub fn read_str(text: String) -> Result<MalForm, MalErr> {
    read_form(&mut Reader {
        tokens: tokenize(text),
        cursor: 0,
    })
}

type MalFunction = dyn Fn(Vec<MalForm>) -> MalForm;

#[derive(Clone)]
pub enum MalForm {
    List(Vec<MalForm>),
    Vector(Vec<MalForm>),
    Int(i64),
    Nil,
    Bool(bool),
    Symbol(String),
    String(String),
    Function(Rc<MalFunction>),
}

impl std::fmt::Debug for MalForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Vector(arg0) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::Int(arg0) => f.debug_tuple("Int").field(arg0).finish(),
            Self::Nil => write!(f, "Nil"),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Function(_arg0) => f.debug_tuple("Function").field(&"f").finish(),
        }
    }
}

impl PartialEq for MalForm {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Vector(l0), Self::Vector(r0)) => l0 == r0,
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Function(_l0), Self::Function(_r0)) => false,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

fn read_form(r: &mut Reader) -> Result<MalForm, MalErr> {
    if r.eof() {
        Err(String::from("Unexpected end of input!"))
    } else if r.peek() == "(" {
        Ok(MalForm::List(read_list(r, ")")?))
    } else if r.peek() == "[" {
        Ok(MalForm::Vector(read_list(r, "]")?))
    } else {
        read_atom(r)
    }
}

fn read_list(r: &mut Reader, end_token: &str) -> Result<Vec<MalForm>, MalErr> {
    let mut v: Vec<MalForm> = vec![];
    r.next();
    while !r.eof() && r.peek() != end_token {
        v.push(read_form(r)?)
    }
    if r.eof() {
        return Err(String::from("unbalanced"));
    }
    r.next();
    Ok(v)
}

fn read_atom(r: &mut Reader) -> Result<MalForm, MalErr> {
    let t = r.next();
    let c = t.chars().nth(0).unwrap();
    if Regex::new(r"^[+-]?\d+$").unwrap().is_match(t) {
        Ok(MalForm::Int(t.parse::<i64>().unwrap()))
    } else if c.is_alphabetic() || "+-*/".contains(c) {
        Ok(match t {
            "nil" => MalForm::Nil,
            "true" => MalForm::Bool(true),
            "false" => MalForm::Bool(false),
            _ => MalForm::Symbol(String::from(t)),
        })
    } else if c == '"' {
        if t.len() >= 2 && t.ends_with("\"") {
            Ok(MalForm::String(String::from(&t[1..t.len() - 1])))
        } else {
            Err(String::from("unbalanced"))
        }
    } else {
        Err(format!("unknown token: {t}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_test() {
        assert_eq!(
            tokenize(String::from(", ( + 1, 2, ) ,")),
            vec!["(", "+", "1", "2", ")"]
        );
        assert_eq!(
            tokenize(String::from(r#"(str "he(ll)o" "world")"#)),
            vec!["(", "str", r#""he(ll)o""#, r#""world""#, ")"]
        );
        assert_eq!(tokenize(String::from(r#""okay""#)), vec![r#""okay""#]);
        assert_eq!(tokenize(String::from(r#""okay"#)), vec![r#""okay"#]);
        assert_eq!(tokenize(String::from(r#"okay""#)), vec!["okay", "\""]);
        assert_eq!(
            tokenize(String::from(r#""ok"ay""#)),
            vec![r#""ok""#, "ay", "\""]
        );
        assert_eq!(tokenize(String::from("\"")), vec!["\""]);
    }
    #[test]
    fn read_test() {
        assert_eq!(
            read_str(String::from("(+ 1 2)")),
            Ok(MalForm::List(vec![
                MalForm::Symbol(String::from("+")),
                MalForm::Int(1),
                MalForm::Int(2)
            ]))
        );
        assert_eq!(
            read_str(String::from("(+ 17 (* 25 36))")),
            Ok(MalForm::List(vec![
                MalForm::Symbol(String::from("+")),
                MalForm::Int(17),
                MalForm::List(vec![
                    MalForm::Symbol(String::from("*")),
                    MalForm::Int(25),
                    MalForm::Int(36)
                ])
            ])),
        );
    }
}
