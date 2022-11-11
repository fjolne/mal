use std::collections::HashMap;

use crate::reader::MalForm;

pub struct Env<'a> {
    pub outer: Option<&'a Env<'a>>,
    data: HashMap<String, MalForm>,
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Env {
            outer: None,
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, k: &str, v: MalForm) -> &MalForm {
        self.data.insert(k.to_owned(), v);
        self.data.get(k).unwrap()
    }

    fn find(&self, k: &str) -> Option<&Self> {
        if self.data.contains_key(k) {
            Some(self)
        } else if let Some(ref env) = self.outer {
            env.find(k)
        } else {
            None
        }
    }

    pub fn get(&self, k: &str) -> Option<&MalForm> {
        if let Some(env) = self.find(k) {
            Some(env.data.get(k).unwrap())
        } else {
            None
        }
    }
}

impl<'a> From<HashMap<String, MalForm>> for Env<'a> {
    fn from(data: HashMap<String, MalForm>) -> Self {
        Env { outer: None, data }
    }
}
