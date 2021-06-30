use crate::bindings::Bindings;
use crate::bitstring::BitString;
use crate::bytecode::{Bytecode, Instruction};
use crate::callable::Callable;
use crate::coded_function::CodedFunction;
use crate::pattern::PatternParseMulti;
use crate::value::Value;
use itertools::Itertools;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("No variant of function `{func_name}` matches the argument list: {args:?}")]
    NoMatch { func_name: String, args: Vec<Value> },
    #[error("Task stack is empty")]
    TaskStackEmpty,
    #[error("Value stack was expected to be of the size {expected}, but is of the size {real}")]
    ValueStackSizeUnexpected { real: usize, expected: usize },
    #[error("Value stack is empty")]
    ValueStackEmpty,
    #[error("No such variable or function: `{name}`")]
    VariableNotFound { name: String },
    #[error("Value is not callable")]
    NotCallable,
    #[error("Value is not a bit string")]
    NotBitString,
}

pub type BasicExecResult<T> = Result<T, ExecError>;
pub type ExecResult = BasicExecResult<()>;

pub struct VM {
    global_bindings: Bindings,
    task_stack: Vec<Task>,
}

impl VM {
    pub fn new(global_bindings: Bindings) -> VM {
        VM {
            global_bindings,
            task_stack: Vec::new(),
        }
    }

    pub fn invoke_by_name(&mut self, name: &str, arguments: Vec<Value>) -> ExecResult {
        let callable = self
            .global_bindings
            .get_value(name)
            .map(Clone::clone)
            .ok_or_else(|| ExecError::VariableNotFound {
                name: String::from(name),
            })?
            .into_callable()
            .ok_or_else(|| ExecError::NotCallable)?;

        self.invoke(callable, arguments, BitString::empty(), BitString::empty())
    }

    pub fn invoke(
        &mut self,
        callable: Callable,
        arguments: Vec<Value>,
        prepend: BitString,
        append: BitString,
    ) -> ExecResult {
        match callable {
            Callable::Coded(coded_function) => {
                self.task_stack
                    .push(make_task(coded_function, arguments, prepend, append)?);
            }
            Callable::Native(native_function) => {
                let ret = (native_function.func)(arguments)?;
                self.task_stack
                    .last_mut()
                    .ok_or(ExecError::TaskStackEmpty)?
                    .execution_state
                    .value_stack
                    .push(ret)
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> ExecResult {
        let current_task = self
            .task_stack
            .last_mut()
            .ok_or(ExecError::TaskStackEmpty)?;
        let step_result = current_task.step(&self.global_bindings)?;

        match step_result {
            StepResult::Nothing => (),
            StepResult::Call {
                callable,
                arguments,
                tail,
            } => match tail {
                TailStatus::NotTail => {
                    self.invoke(callable, arguments, BitString::empty(), BitString::empty())?
                }
                TailStatus::Tail { prepends, appends } => {
                    let current_task = self.task_stack.pop().unwrap();
                    let prepend = current_task
                        .prepend
                        .into_iter()
                        .chain(prepends.into_iter().map(|x| x.into_iter()).flatten())
                        .collect();
                    let append = appends
                        .into_iter()
                        .map(|x| x.into_iter())
                        .flatten()
                        .chain(current_task.append.into_iter())
                        .collect();
                    self.invoke(callable, arguments, prepend, append)?;
                }
            },
            StepResult::FinishTask { return_value } => {
                let current_task = self.task_stack.pop().unwrap();
                let pushed_value = match return_value {
                    Value::BitString(s) => current_task
                        .prepend
                        .into_iter()
                        .chain(s.into_iter())
                        .chain(current_task.append.into_iter())
                        .collect::<BitString>()
                        .into(),
                    Value::Callable(c) => {
                        if !current_task.prepend.is_empty() || !current_task.append.is_empty() {
                            return Err(ExecError::NotBitString);
                        }
                        c.into()
                    }
                };

                println!("Return: {:?}", pushed_value);
                self.task_stack
                    .last_mut()
                    .ok_or(ExecError::TaskStackEmpty)?
                    .push(pushed_value);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ExecutionState {
    value_stack: Vec<Value>,
    cursor: usize,
}

impl ExecutionState {
    pub fn new() -> ExecutionState {
        ExecutionState {
            value_stack: Vec::new(),
            cursor: 0,
        }
    }
}

#[derive(Debug)]
enum TailStatus {
    NotTail,
    Tail {
        appends: Vec<BitString>,
        prepends: Vec<BitString>,
    },
}

#[derive(Debug)]
enum StepResult {
    Nothing,
    FinishTask {
        return_value: Value,
    },
    Call {
        callable: Callable,
        arguments: Vec<Value>,
        tail: TailStatus,
    },
}

#[derive(Debug)]
struct Task {
    bytecode: Bytecode,
    local_bindings: Bindings,
    execution_state: ExecutionState,
    prepend: BitString,
    append: BitString,
}

impl Task {
    fn step(&mut self, global_bindings: &Bindings) -> BasicExecResult<StepResult> {
        let instruction = match self.current_instruction() {
            Some(x) => x,
            None => {
                return Ok(StepResult::FinishTask {
                    return_value: self.return_value()?,
                })
            }
        }
        .clone();

        println!("{:?}", instruction);

        self.execution_state.cursor += 1;
        Ok(match instruction {
            Instruction::LoadConst(bit_string) => {
                self.push(bit_string.clone().into());
                StepResult::Nothing
            }
            Instruction::LoadVar { name } => {
                let var = self
                    .get_var(&name, global_bindings)
                    .ok_or_else(|| ExecError::VariableNotFound { name: name.clone() })?;

                self.push(var);
                StepResult::Nothing
            }
            Instruction::Trampoline => {
                let value = self.pop_result()?;
                match value {
                    Value::Callable(Callable::Coded(f)) if f.is_trampoline_callable() => {
                        self.execution_state.cursor -= 1;
                        StepResult::Call {
                            callable: f.into(),
                            arguments: Vec::new(),
                            tail: TailStatus::NotTail,
                        }
                    }
                    value => {
                        self.push(value);
                        StepResult::Nothing
                    }
                }
            }
            Instruction::Call(num_args) => {
                let arguments = self.pop_n_result(num_args)?;
                let callable = self
                    .pop_result()?
                    .into_callable()
                    .ok_or(ExecError::NotCallable)?;
                StepResult::Call {
                    callable,
                    arguments,
                    tail: TailStatus::NotTail,
                }
            }
            Instruction::Cat(num_children) => {
                let children: Vec<_> = self
                    .pop_n_result(num_children)?
                    .into_iter()
                    .map(|x| x.into_bit_string().ok_or(ExecError::NotBitString))
                    .try_collect()?;

                let bit_string: BitString = children
                    .into_iter()
                    .map(|x| x.into_iter())
                    .flatten()
                    .collect();

                self.execution_state.value_stack.push(bit_string.into());
                StepResult::Nothing
            }
            Instruction::Tail { prepend, append } => {
                let num_args = self
                    .execution_state
                    .value_stack
                    .len()
                    .checked_sub(prepend + append + 1)
                    .ok_or(ExecError::ValueStackEmpty)?;

                dbg!(num_args);

                let arguments = self.pop_n_result(num_args)?;

                let callable = self
                    .pop_result()?
                    .into_callable()
                    .ok_or(ExecError::NotCallable)?;

                dbg!("here");

                let appends = self
                    .pop_n_result(append)?
                    .into_iter()
                    .map(|x| x.into_bit_string().ok_or(ExecError::NotBitString))
                    .try_collect()?;

                let prepends = self
                    .pop_n_result(prepend)?
                    .into_iter()
                    .map(|x| x.into_bit_string().ok_or(ExecError::NotBitString))
                    .try_collect()?;

                if !self.execution_state.value_stack.is_empty() {
                    return Err(ExecError::ValueStackSizeUnexpected {
                        real: self.execution_state.value_stack.len(),
                        expected: 0,
                    });
                }

                StepResult::Call {
                    callable,
                    arguments,
                    tail: TailStatus::Tail { prepends, appends },
                }
            }
        })
    }

    fn current_instruction(&self) -> Option<&Instruction> {
        self.bytecode.at(self.execution_state.cursor)
    }

    fn return_value(&self) -> BasicExecResult<Value> {
        let stack = &self.execution_state.value_stack;
        if stack.len() == 1 {
            Ok(stack[0].clone())
        } else {
            Err(ExecError::ValueStackSizeUnexpected {
                real: stack.len(),
                expected: 1,
            })
        }
    }

    fn get_var(&self, name: &str, global_bindings: &Bindings) -> Option<Value> {
        self.local_bindings
            .get_value(name)
            .or_else(|| global_bindings.get_value(name))
            .map(|x| x.clone())
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

    fn pop_n(&mut self, n: usize) -> Option<Vec<Value>> {
        let len = self.execution_state.value_stack.len();
        let num_remaining_items = len.checked_sub(n)?;
        Some(
            self.execution_state
                .value_stack
                .split_off(num_remaining_items),
        )
    }

    fn pop_n_result(&mut self, n: usize) -> BasicExecResult<Vec<Value>> {
        self.pop_n(n).ok_or(ExecError::ValueStackEmpty)
    }
}

fn make_task(
    coded_function: CodedFunction,
    arguments: Vec<Value>,
    prepend: BitString,
    append: BitString,
) -> BasicExecResult<Task> {
    for var in coded_function.variants {
        let local_bindings = match var.patterns.parse(arguments.clone()) {
            Some(x) => x,
            None => continue,
        };
        let bytecode = var.body;

        return Ok(Task {
            bytecode,
            local_bindings,
            execution_state: ExecutionState::new(),
            prepend,
            append,
        });
    }

    Err(ExecError::NoMatch {
        func_name: coded_function.name,
        args: arguments,
    })
}
