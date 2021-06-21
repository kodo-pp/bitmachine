use std::collections::HashMap;
use std::rc::Rc;
use crate::program::{Functional, Function};
use thiserror::Error;

pub type FunctionMap = HashMap<String, Function>;

#[derive(Debug)]
pub struct Executor {
    functions: HashMap<String, Rc<Functional>>,
}

impl Executor {
    pub fn new() -> Self {
        Self { functions: HashMap::new() }
    }

    pub fn add_functional(&mut self, name: String, func: Rc<Functional>) {
        self.functions.insert(name, func);
    }

    pub fn get_functional(&self, name: &str) -> Option<&Rc<Functional>> {
        self.functions.get(name)
    }
}

impl From<FunctionMap> for Executor {
    fn from(map: FunctionMap) -> Self {
        let mut executor = Self::new();
        for (name, func) in map {
            executor.add_functional(name, Rc::new(Functional::Defined(func)));
        }
        executor
    }
}

#[derive(Debug, Error)]
#[error("Execution error: {message}")]
pub struct ExecutionError {
    pub message: String,
}

impl From<String> for ExecutionError {
    fn from(message: String) -> Self {
        Self { message }
    }
}
