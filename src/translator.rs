use crate::ast::{Expr, Program, FunctionMap, Function, FunctionVariant};
use crate::bytecode::{Bytecode, Instruction};
use crate::compiled::{Program as CompiledProgram, FunctionMap as CompiledFunctionMap};
use crate::coded_function::{CodedFunction, CodedFunctionVariant};
use std::iter;

pub trait Compile {
    fn compile(self) -> CompiledProgram;
}

pub trait ToBytecode {
    fn to_bytecode(self) -> Bytecode;
}

pub trait ToInstructions {
    fn to_instructions(self, call_status: CallStatus) -> Vec<Instruction>;
}

impl<T: ToInstructions> ToBytecode for T {
    fn to_bytecode(self) -> Bytecode {
        Bytecode::new(self.to_instructions(CallStatus::Tail { prepend: 0, append: 0 }))
    }
}

impl Compile for Program {
    fn compile(self) -> CompiledProgram {
        CompiledProgram {
            function_map: compile_function_map(self.function_map),
        }
    }
}

fn compile_function_map(function_map: FunctionMap) -> CompiledFunctionMap {
    function_map.into_iter().map(|(name, func)| (name, compile_function(func).into())).collect()
}

fn compile_function(func: Function) -> CodedFunction {
    CodedFunction {
        variants: func.variants.into_iter().map(compile_function_variant).collect()
    }
}

fn compile_function_variant(var: FunctionVariant) -> CodedFunctionVariant {
    CodedFunctionVariant { patterns: var.patterns, body: var.body.to_bytecode() }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CallStatus {
    Regular,
    Tail { prepend: usize, append: usize },
}

impl ToInstructions for Expr {
    fn to_instructions(self, call_status: CallStatus) -> Vec<Instruction> {
        match self {
            Expr::Variable { name, trampoline } => vec![Instruction::LoadVar { name, trampoline }],
            Expr::Literal(bit_string) => vec![Instruction::LoadConst(bit_string)],
            Expr::Call { callee, args } => function_call_to_instructions(*callee, args, call_status),
            Expr::Cat { children } => concatenation_to_instructions(children, call_status),
        }
    }
}

fn function_call_to_instructions(
    callee: Expr,
    args: Vec<Expr>,
    call_status: CallStatus,
) -> Vec<Instruction> {
    let mut instructions = callee.to_instructions(CallStatus::Regular);

    let len = args.len();
    
    for arg in args.into_iter() {
        instructions.append(&mut arg.to_instructions(CallStatus::Regular));
    }
    

    instructions.push(match call_status {
        CallStatus::Regular => Instruction::Call(len),
        CallStatus::Tail { prepend, append } => Instruction::Tail { prepend, append },
    });

    instructions
}

fn concatenation_to_instructions(children: Vec<Expr>, call_status: CallStatus) -> Vec<Instruction> {
    match (find_single_complex_expr(&children), call_status) {
        (Some(i), CallStatus::Tail { prepend, append }) => {
            let (prepends, focus, appends) = split_off_3(children, i);
            let prepends_len = prepends.len();
            let appends_len = appends.len();

            prepends.into_iter()
                .chain(appends.into_iter())
                .map(|expr| expr.to_instructions(CallStatus::Regular))
                .flatten()
                .chain(focus.to_instructions(CallStatus::Tail {
                    prepend: prepends_len + prepend,
                    append: appends_len + append,
                }))
                .collect()
        }
        _ => {
            let children_len = children.len();

            children
                .into_iter()
                .map(|expr| expr.to_instructions(CallStatus::Regular))
                .flatten()
                .chain(iter::once(Instruction::Cat(children_len)))
                .collect()
        }
    }
}

fn split_off_3<T>(mut vec: Vec<T>, i: usize) -> (Vec<T>, T, Vec<T>) {
    let mut middle_and_right = vec.split_off(i);
    let left = vec;
    let right = middle_and_right.split_off(1);
    let middle = middle_and_right;

    let middle_element = middle.into_iter().next().unwrap();
    (left, middle_element, right)
}

trait ExprClassification {
    fn is_complex(&self) -> bool;
}

impl ExprClassification for Expr {
    fn is_complex(&self) -> bool {
        match self {
            Expr::Call { .. } => true,
            _ => false,
        }
    }
}

fn find_single_complex_expr(exprs: &[Expr]) -> Option<usize> {
    let mut last_complex = None;

    for (i, expr) in exprs.iter().enumerate() {
        match (last_complex, expr.is_complex()) {
            (_, false) => (),
            (None, true) => { last_complex = Some(i); },
            (Some(_), true) => return None,
        }
    }

    last_complex
}
