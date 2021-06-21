use crate::bitstring::BitString;
use std::iter::Iterator;

#[derive(Debug)]
pub struct Bytecode {
    instructions: Vec<Instruction>,
}

impl Bytecode {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &Instruction> {
        self.instructions.iter()
    }
}

#[derive(Debug)]
pub enum Instruction {
    LoadConst(BitString),
    LoadVar { name: String, trampoline: bool },
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
            Instruction::LoadVar { name, trampoline } => format!(
                "load_name{} {:?}",
                if *trampoline { ".nt" } else { "" },
                name
            ),
            Instruction::Call(n) => format!("call {}", n),
            Instruction::Cat(n) => format!("cat {}", n),
            Instruction::Tail { prepend, append } => {
                format!("tail [pre = {}], [app = {}]", prepend, append)
            }
        }
    }
}
