use std::collections::HashMap;

use crate::{
    ops::{BinaryOp, UnaryOp},
    Primitive, Table, Value,
};

#[derive(Debug)]
pub enum RuntimeError {
    EmptyStack,
    NoStackFrames,
    InvalidVariable,
    NotATable,
    InvalidTableKey,
    InvalidProgramCounter,
}

type Vars = HashMap<String, Value>;

struct StackFrame {
    locals: Vars,
    stack: Vec<Value>,
}

struct State {
    pc: usize,
    stack_frames: Vec<StackFrame>,
}

impl State {
    fn new() -> Self {
        State {
            pc: 0,
            stack_frames: Vec::new(),
        }
    }

    fn incr_pc(&mut self) {
        self.pc += 1;
    }

    fn current_frame(&mut self) -> Result<&mut StackFrame, RuntimeError> {
        self.stack_frames
            .last_mut()
            .ok_or(RuntimeError::NoStackFrames)
    }

    fn push_frame(&mut self, locals: Vars) {
        self.stack_frames.push(StackFrame {
            locals,
            stack: Vec::new(),
        });
    }

    fn push_stack(&mut self, value: Value) -> Result<(), RuntimeError> {
        self.current_frame()?.stack.push(value);
        Ok(())
    }

    fn pop_stack(&mut self) -> Result<Value, RuntimeError> {
        self.current_frame()?
            .stack
            .pop()
            .ok_or(RuntimeError::EmptyStack)
    }

    fn peek_stack(&mut self) -> Result<&Value, RuntimeError> {
        self.current_frame()?
            .stack
            .last()
            .ok_or(RuntimeError::EmptyStack)
    }

    fn get_var(&mut self, name: &str) -> Result<Value, RuntimeError> {
        self.stack_frames
            .iter()
            .rev()
            .find_map(|frame| frame.locals.get(name))
            .cloned()
            .ok_or(RuntimeError::InvalidVariable)
    }

    fn set_var(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        self.current_frame()?.locals.insert(name.into(), value);
        Ok(())
    }
}

enum Instruction {
    Nop,

    Copy,
    Swap(usize),
    Pop,

    /// TOS = op TOS
    UnaryOp(UnaryOp),
    /// TOS = TOS1 op TOS
    BinaryOp(BinaryOp),

    /// Store TOS1 into value at TOS.
    ///
    /// `var[TOS] = TOS1`
    StoreName,

    /// Pop the value at the top of the stack, and store the given const in it.
    ///
    /// `var[TOS] = const`
    StorePrimitive(Primitive),

    /// TOS = var[TOS]
    PushName,
    /// Push the given constant onto the stack.
    PushPrimitive(Primitive),

    /// TOS = TOS1[TOS]
    TableGet,
    TableListBuild(usize),
    TableDictBuild(usize),
    TableMerge,

    /// if TOS is true, jump forwards by the given amount
    PopJumpIfTrue(usize),
    /// if TOS is false, jump forwards by the given amount
    PopJumpIfFalse(usize),
    /// jump forwards by the given amount
    Jump(usize),

    PushFunction(usize),
    /// TOS = TOS(TOS1)
    Call,
    /// return TOS
    Return,
}

impl Instruction {
    fn eval(&self, state: &mut State) -> Result<bool, RuntimeError> {
        use Instruction::*;
        match self {
            Nop => {
                state.incr_pc();
                Ok(false)
            }
            Copy => {
                state.incr_pc();
                let value = state.peek_stack()?.clone();
                state.push_stack(value)?;
                Ok(false)
            }
            StoreName => {
                state.incr_pc();
                let value = state.pop_stack()?;
                let name = state.pop_stack()?;
                let var = name
                    .get_value::<String>()
                    .ok_or(RuntimeError::InvalidVariable)?;
                state.set_var(&var, value)?;
                Ok(false)
            }
            StorePrimitive(value) => {
                state.incr_pc();
                let name = state.pop_stack()?;
                let var = name
                    .get_value::<String>()
                    .ok_or(RuntimeError::InvalidVariable)?;
                state.set_var(&var, value.clone().into())?;
                Ok(false)
            }
            PushName => {
                state.incr_pc();
                let name = state.pop_stack()?;
                let var = name
                    .get_value::<String>()
                    .ok_or(RuntimeError::InvalidVariable)?;
                let value = state.get_var(&var)?;
                state.push_stack(value)?;
                Ok(false)
            }
            PushPrimitive(value) => {
                state.incr_pc();
                state.push_stack(value.clone().into())?;
                Ok(false)
            }
            TableGet => {
                state.incr_pc();
                let key = state.pop_stack()?;
                let tos = state.peek_stack()?;
                match tos {
                    Value::Table(table) => {
                        let value = table
                            .get(key.try_into().map_err(|_| RuntimeError::InvalidVariable)?)
                            .cloned()
                            .unwrap_or(Value::nil());
                        state.push_stack(value)?;
                        Ok(false)
                    }
                    _ => Err(RuntimeError::InvalidVariable),
                }
            }
            TableListBuild(n) => {
                state.incr_pc();
                let mut table = Table::new();
                for i in 0..*n {
                    let value = state.pop_stack()?;
                    table.set(i, value);
                }
                Ok(false)
            }
            TableDictBuild(n) => {
                state.incr_pc();
                let mut table = Table::new();
                for _ in 0..*n {
                    let key = state
                        .pop_stack()?
                        .try_into()
                        .map_err(|_| RuntimeError::InvalidTableKey)?;
                    let value = state.pop_stack()?;
                    table.set(key, value);
                }
                Ok(false)
            }
            TableMerge => {
                state.incr_pc();
                let tos: Table = state
                    .pop_stack()?
                    .try_into()
                    .map_err(|_| RuntimeError::NotATable)?;
                let mut tos1: Table = state
                    .pop_stack()?
                    .try_into()
                    .map_err(|_| RuntimeError::NotATable)?;

                for (key, value) in tos {
                    tos1.set(key, value);
                }

                state.push_stack(tos1.into())?;
                Ok(false)
            }
            Return => {
                if state.stack_frames.len() == 1 {
                    return Ok(true);
                }

                // TODO: implement popping the stack frame and jumping to the return address
                Ok(false)
            }
            _ => todo!(),
        }
    }
}

pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub(crate) fn new() -> Self {
        Program {
            instructions: Vec::new(),
        }
    }

    fn push_function(&mut self, name: &str, body: impl FnOnce(&mut Program)) {
        let mut p = Program::new();
        body(&mut p);
        use Instruction::*;
        self.instructions.push(PushFunction(3));
        self.instructions.push(PushPrimitive(name.into()));
        self.instructions.push(StoreName);
        self.instructions.push(Jump(p.instructions.len() + 1));
        self.instructions.extend(p.instructions);
    }

    pub fn run(&self) -> Result<Value, RuntimeError> {
        self.run_with(HashMap::new())
    }

    pub fn run_with(&self, globals: Vars) -> Result<Value, RuntimeError> {
        let mut state = State {
            pc: 0,
            stack_frames: vec![StackFrame {
                locals: globals,
                stack: Vec::new(),
            }],
        };

        loop {
            if let Some(instruction) = self.instructions.get(state.pc) {
                let should_stop = instruction.eval(&mut state)?;
                if should_stop {
                    return Ok(state.pop_stack()?);
                }
            } else {
                // only if pc points exactly to the end of the program
                // should we return nil, otherwise the pc is invalid
                if state.pc == self.instructions.len() {
                    return Ok(Value::nil());
                } else {
                    return Err(RuntimeError::InvalidProgramCounter);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Instruction::*;
    use super::*;

    #[test]
    fn test_nop() {
        let mut state = State {
            pc: 0,
            stack_frames: Vec::new(),
        };

        assert_eq!(Nop.eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
    }

    #[test]
    fn test_copy() {
        let mut state = State {
            pc: 0,
            stack_frames: vec![StackFrame {
                locals: HashMap::new(),
                stack: vec![1.into()],
            }],
        };

        assert_eq!(Copy.eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().get_primitive(),
            Some(1.into())
        );
        assert_eq!(
            state.stack_frames[0].stack[1].clone().get_primitive(),
            Some(1.into())
        );
    }
}
