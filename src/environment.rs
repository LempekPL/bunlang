use std::collections::HashMap;
use crate::interpreter::Value;

#[derive(Debug)]
pub struct Environment {
    pub vars: HashMap<String, Value>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Environment>) -> Self {
        Self {
            vars: HashMap::new(),
            parent: parent.map(Box::new),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.vars.get(name) {
            Some(value) => Some(value.clone()),
            None => {
                if let Some(parent) = &self.parent {
                    parent.get(name)
                } else {
                    None
                }
            }
        }
    }

    pub fn set(&mut self, name: impl Into<String>, val: Value) {
        self.vars.insert(name.into(), val);
    }
}