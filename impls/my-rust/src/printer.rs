use crate::reader::MalForm;

pub fn pr_str(form: &MalForm) -> String {
    fn pr_seq(v: &Vec<MalForm>) -> String {
        v.iter()
            .map(|x| pr_str(x))
            .collect::<Vec<String>>()
            .join(" ")
    }
    match form {
        MalForm::List(v) => format!("({})", pr_seq(&v)),
        MalForm::Vector(v) => format!("[{}]", pr_seq(&v)),
        MalForm::Bool(b) => String::from(if *b == true { "true" } else { "false" }),
        MalForm::Int(n) => n.to_string(),
        MalForm::Nil => String::from("nil"),
        MalForm::Symbol(s) => s.clone(),
        MalForm::String(s) => format!(r#""{s}""#),
        MalForm::Function(_) => "<function>".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::read_str;

    use super::*;

    #[test]
    fn pr_str_test() {
        let text = "(+ 17 (* 25 36))";
        assert_eq!(pr_str(&read_str(String::from(text)).unwrap()), text);
    }
}
