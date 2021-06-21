use crate::bindings::Bindings;
use crate::value::Value;
use crate::bytecode::{Bytecode, Instruction};
use crate::callable::Callable;
use crate::coded_function::CodedFunction;
use crate::pattern::PatternParseMulti;

#[derive(Debug)]
pub enum ExecError {
    NoMatch { func_name: String, args: Vec<Value> },
    TaskStackEmpty,
    ValueStackSizeNotOne { size: usize },
    ValueStackEmpty,
    VariableNotFound { name: String },
}

pub type BasicExecResult<T> = Result<T, ExecError>;
pub type ExecResult = BasicExecResult<()>;

pub struct VM {
    global_bindings: Bindings,
    task_stack: Vec<Task>,
}

impl VM {
    pub fn new(global_bindings: Bindings) -> VM {
        VM { global_bindings, task_stack: Vec::new() }
    }

    pub fn invoke(&mut self, callable: Callable, arguments: Vec<Value>) -> ExecResult {
        match callable {
            Callable::Coded(coded_function) => {
                self.task_stack.push(make_task(coded_function, arguments)?);
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> ExecResult {
        let current_task = self.task_stack.last().ok_or(ExecError::TaskStackEmpty)?;
        current_task.step(&self)
    }
}

#[derive(Debug)]
struct ExecutionState {
    value_stack: Vec<Value>,
    cursor: usize,
}

impl ExecutionState {
    pub fn new() -> ExecutionState {
        ExecutionState { value_stack: Vec::new(), cursor: 0 }
    }
}

#[derive(Debug)]
enum StepResult {
    Nothing,
    FinishTask { return_value: Value },
    Call { callable: Callable, arguments: Vec<Value>, tail: bool },
}

#[derive(Debug)]
struct Task {
    bytecode: Bytecode,
    local_bindings: Bindings,
    execution_state: ExecutionState,
}

impl Task {
    fn step(&mut self, vm: &VM) -> BasicExecResult<StepResult> {
        let instruction = match self.current_instruction() {
            Some(x) => x,
            None => return Ok(StepResult::FinishTask {
                return_value: self.return_value(),
            }),
        };

        self.execution_state.cursor += 1;
        Ok(match instruction {
            Instruction::LoadConst(bit_string) => {
                self.push(bit_string.clone().into());
                StepResult::Nothing
            }
            Instruction::LoadVar { name } => {
                let var = self
                    .get_var(name)
                    .ok_or_else(|| ExecError::VariableNotFound { name: name.clone() })?;

                self.push(var);
                StepResult::Nothing
            }
            Instruction::Trampoline => {
                let value = self.pop_result()?;
                match value {
                    Value::Callable(callable) => {
                        self.execution_state.cursor -= 1;
                        StepResult::Call { callable, arguments: Vec::new(), tail: false }
                    }
                    value => {
                        self.push(value);
                        StepResult::Nothing
                    }
                }
            }
            Instruction::Call(num_args) => {
                // TODO
            }
        })
    }

    fn current_instruction(&self) -> Option<&Instruction> {
        self.bytecode.at(self.execution_state.cursor)
    }

    fn return_value(&self) -> Value {
        todo!();
    }

    fn get_var(&self, name: &str) -> Option<Value> {
        todo!();
    }

    fn push(&mut self, value: Value) {
        self.execution_state.value_stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.execution_state.value_stack.pop()
    }

    fn pop_result(&mut self) -> BasicExecResult<Value> {
        self.pop().ok_or(ExecError::ValueStackEmpty)
    }
}

fn make_task(coded_function: CodedFunction, arguments: Vec<Value>) -> BasicExecResult<Task> {
    for var in coded_function.variants {
        let local_bindings = match var.patterns.parse(arguments.clone()) {
            Some(x) => x,
            None => continue, 
        };
        let bytecode = var.body;

        return Ok(Task { bytecode, local_bindings, execution_state: ExecutionState::new() });
    }

    Err(ExecError::NoMatch { func_name: coded_function.name, args: arguments })
}
