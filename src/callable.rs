use crate::coded_function::CodedFunction;

#[derive(Debug, Clone)]
pub enum Callable {
    Coded(CodedFunction),
}

impl From<CodedFunction> for Callable {
    fn from(func: CodedFunction) -> Callable {
        Callable::Coded(func)
    }
}
