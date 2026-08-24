use std::collections::HashMap;
use crate::interpreter::ControlFlow;
use crate::value::Value;

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
        if let Some(val) = self.vars.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn get_top(&self) -> HashMap<String, Value> {
        let mut curr = self;
        while let Some(ref parent) = curr.parent {
            curr = parent;
        }
        curr.vars.clone()
    }

    pub fn get_all(&self) -> HashMap<String, Value> {
        let mut all = HashMap::new();
        if let Some(parent) = &self.parent {
            all.extend(parent.get_all());
        }
        for (k, v) in &self.vars {
            all.insert(k.clone(), v.clone());
        }
        all
    }

    pub fn define(&mut self, name: impl Into<String>, val: Value) {
        self.vars.insert(name.into(), val);
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), String> {
        if let Some(existing) = self.vars.get(name) {
            if std::mem::discriminant(existing) == std::mem::discriminant(&val) {
                self.vars.insert(name.to_string(), val);
                Ok(())
            } else {
                Err(format!(
                    "Type mismatch: cannot assign type '{}' to variable '{}' of type '{}'",
                    val.get_type_name(),
                    name,
                    existing.get_type_name()
                ))
            }
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, val)
        } else {
            Err(format!("Undefined variable '{}'", name))
        }
    }
}