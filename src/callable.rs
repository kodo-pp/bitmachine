use crate::coded_function::CodedFunction;
use crate::native_function::NativeFunction;

#[derive(Debug, Clone)]
pub enum Callable {
    Coded(CodedFunction),
    Native(NativeFunction)
}

impl From<CodedFunction> for Callable {
    fn from(func: CodedFunction) -> Callable {
        Callable::Coded(func)
    }
}
