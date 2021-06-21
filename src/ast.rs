use crate::bitstring::BitString;
use crate::pattern::MultiPattern;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Program {
    pub function_map: FunctionMap,
}

pub type FunctionMap = HashMap<String, Function>;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub variants: Vec<FunctionVariant>,
}

#[derive(Debug)]
pub struct FunctionVariant {
    pub patterns: MultiPattern,
    pub body: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Variable { name: String, trampoline: bool },
    Literal(BitString),
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Cat { children: Vec<Expr> },
}
