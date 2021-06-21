use crate::bitstring::BitString;
use std::iter::Iterator;

#[derive(Debug, Clone)]
pub struct Bytecode {
    instructions: Vec<Instruction>,
}

impl Bytecode {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Instruction> {
        self.instructions.iter()
    }

    pub fn at(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(BitString),
    LoadVar { name: String },
    Trampoline,
    Call(usize),
    Cat(usize),
    Tail { prepend: usize, append: usize },
}

pub trait Pretty {
    fn pretty(&self) -> String;
}

impl Pretty for Instruction {
    fn pretty(&self) -> String {
        match self {
            Instruction::LoadConst(s) => format!("load_const {:?}", s),
            Instruction::LoadVar { name } => format!("load_name {:?}", name),
            Instruction::Trampoline => String::from("trampoline"),
            Instruction::Call(n) => format!("call {}", n),
            Instruction::Cat(n) => format!("cat {}", n),
            Instruction::Tail { prepend, append } => {
                format!("tail [pre = {}], [app = {}]", prepend, append)
            }
        }
    }
}
