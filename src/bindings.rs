use crate::value::Value;
use std::collections::HashMap;

pub struct Bindings(HashMap<String, Value>);

impl Bindings {
    pub fn new(bindings: HashMap<String, Value>) -> Self {
        Self(bindings)
    }

    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn get_map(&self) -> &HashMap<String, Value> {
        &self.0
    }

    pub fn into_map(self) -> HashMap<String, Value> {
        self.0
    }

    pub fn union_with(mut self, other: Bindings) -> Self {
        self.0.extend(other.into_map().into_iter());
        self
    }

    pub fn add(&mut self, name: String, value: Value) {
        self.0.insert(name, value);
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }
}
