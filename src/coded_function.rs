use crate::bytecode::Bytecode;
use crate::pattern::MultiPattern;

#[derive(Debug)]
pub struct CodedFunction {
    pub variants: Vec<CodedFunctionVariant>,
}

#[derive(Debug)]
pub struct CodedFunctionVariant {
    pub patterns: MultiPattern,
    pub body: Bytecode,
}

