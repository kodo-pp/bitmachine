use crate::callable::Callable;
use crate::bindings::Bindings;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Program {
    pub function_map: FunctionMap,
}

pub type FunctionMap = HashMap<String, Callable>;

impl Into<Bindings> for Program {
    fn into(self) -> Bindings {
        Bindings::new(
            self.function_map
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect()
        )
    }
}
