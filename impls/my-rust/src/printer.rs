use crate::reader::MalForm;

pub fn pr_str(form: &MalForm) -> String {
    match form {
        MalForm::List(v) => format!(
            "({})",
            v.iter()
                .map(|x| pr_str(x))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        MalForm::Bool(b) => String::from(if *b == true { "true" } else { "false" }),
        MalForm::Int(n) => n.to_string(),
        MalForm::Nil => String::from("nil"),
        MalForm::Symbol(s) => s.clone(),
        MalForm::String(s) => format!(r#""{s}""#),
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
