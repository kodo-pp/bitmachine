use crate::callable::Callable;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Program {
    pub function_map: FunctionMap,
}

pub type FunctionMap = HashMap<String, Callable>;
