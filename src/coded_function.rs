use crate::bytecode::Bytecode;
use crate::pattern::MultiPattern;

#[derive(Debug, Clone)]
pub struct CodedFunction {
    pub name: String,
    pub variants: Vec<CodedFunctionVariant>,
}

impl CodedFunction {
    pub fn is_trampoline_callable(&self) -> bool {
        self.variants.iter().any(|x| x.patterns.0.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct CodedFunctionVariant {
    pub patterns: MultiPattern,
    pub body: Bytecode,
}
