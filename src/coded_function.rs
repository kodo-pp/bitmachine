use crate::bytecode::Bytecode;
use crate::pattern::MultiPattern;

#[derive(Debug, Clone)]
pub struct CodedFunction {
    pub name: String,
    pub variants: Vec<CodedFunctionVariant>,
}

#[derive(Debug, Clone)]
pub struct CodedFunctionVariant {
    pub patterns: MultiPattern,
    pub body: Bytecode,
}

