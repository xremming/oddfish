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
    globals: Vars,
    stack_frames: Vec<StackFrame>,
}

impl State {
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

    fn get_var(&self, name: &str) -> Result<Value, RuntimeError> {
        self.stack_frames
            .iter()
            .rev()
            .find_map(|frame| frame.locals.get(name))
            .cloned()
            .or(self.globals.get(name).cloned())
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

    /// `TOS = var[TOS]`
    PushName,
    /// Push the given constant onto the stack.
    PushPrimitive(Primitive),

    /// `TOS = TOS1[TOS]`
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

impl From<UnaryOp> for Instruction {
    fn from(op: UnaryOp) -> Self {
        Instruction::UnaryOp(op)
    }
}

impl From<BinaryOp> for Instruction {
    fn from(op: BinaryOp) -> Self {
        Instruction::BinaryOp(op)
    }
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
                let key = state
                    .pop_stack()?
                    .get_primitive()
                    .ok_or(RuntimeError::InvalidTableKey)?;
                let tos = state.peek_stack()?;
                match tos {
                    Value::Table(table) => {
                        let value = table.get(key).cloned().unwrap_or(Value::nil());
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
                    let key = n - i - 1;
                    table.set(key, value);
                }
                state.push_stack(table.into())?;
                Ok(false)
            }
            TableDictBuild(n) => {
                state.incr_pc();
                let mut table = Table::new();
                for _ in 0..*n {
                    let value = state.pop_stack()?;
                    let key = state
                        .pop_stack()?
                        .get_primitive()
                        .ok_or(RuntimeError::InvalidTableKey)?;
                    table.set(key, value);
                }
                state.push_stack(table.into())?;
                Ok(false)
            }
            TableMerge => {
                state.incr_pc();
                let tos: Table = state
                    .pop_stack()?
                    .get_table()
                    .ok_or(RuntimeError::NotATable)?;
                let mut tos1: Table = state
                    .pop_stack()?
                    .get_table()
                    .ok_or(RuntimeError::NotATable)?;

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
            globals,
            stack_frames: vec![StackFrame {
                locals: HashMap::new(),
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
    use crate::table;

    macro_rules! state {
        (
            $(globals => {$($gk:expr => $gv:expr),* $(,)?})?
            $(locals  => {$($lk:expr => $lv:expr),* $(,)?})?
            $(stack   => [$($sv:expr),*             $(,)?])?
        ) => {
            State {
                pc: 0,
                globals: state!(Vars => {$($($gk => $gv),*)*}),
                stack_frames: vec![StackFrame {
                    locals: state!(Vars => {$($($lk => $lv),*)*}),
                    stack: vec![$($($sv.into()),*)*],
                }],
            }
        };
        (Vars => {$($k:expr => $v:expr),* $(,)?}) => {
            {
                #[allow(unused_mut)]
                let mut vars: Vars = HashMap::new();
                $(vars.insert($k.to_string(), $v.into());)*
                vars
            }
        };
    }

    #[test]
    fn test_get_name_from_globals() {
        let state = state!(
            globals => {
                "x" => 1,
                "y" => 2,
            }
        );

        assert_eq!(state.get_var("x").unwrap(), 1.into());
        assert_eq!(state.get_var("y").unwrap(), 2.into());
    }

    #[test]
    fn test_get_name_from_locals() {
        let state = state!(
            locals => {
                "x" => 1,
                "y" => 2,
            }
        );

        assert_eq!(state.get_var("x").unwrap(), 1.into());
        assert_eq!(state.get_var("y").unwrap(), 2.into());
    }

    #[test]
    fn test_get_name_from_locals_before_globals() {
        let state = state!(
            globals => {
                "x" => 0,
                "y" => 0,
            }
            locals => {
                "x" => 1,
                "y" => 2,
            }
        );

        assert_eq!(state.get_var("x").unwrap(), 1.into());
        assert_eq!(state.get_var("y").unwrap(), 2.into());
    }

    #[test]
    fn test_nop() {
        let mut state = state!();
        assert_eq!(Nop.eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
    }

    #[test]
    fn test_copy() {
        let mut state = state!(
            stack => [1]
        );

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

    #[test]
    fn test_store_name() {
        let mut state = state!(
            globals => { "x" => 0 }
            locals => { "x" => 1 }
            stack => ["x", 2]
        );

        assert_eq!(StoreName.eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 0);
        assert_eq!(state.stack_frames[0].locals.len(), 1);
        assert_eq!(state.stack_frames[0].locals.get("x").unwrap(), &2.into());
    }

    #[test]
    fn test_store_primitive() {
        let mut state = state!(
            globals => { "x" => 0 }
            locals => { "x" => 1 }
            stack => ["x"]
        );

        assert_eq!(StorePrimitive(2.into()).eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 0);
        assert_eq!(state.stack_frames[0].locals.len(), 1);
        assert_eq!(state.stack_frames[0].locals.get("x").unwrap(), &2.into());
    }

    #[test]
    fn test_push_name() {
        let mut state = state!(
            globals => { "x" => 0 }
            locals => { "x" => 1 }
            stack => ["x"]
        );

        assert_eq!(PushName.eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(state.stack_frames[0].stack[0], 1.into());
    }

    #[test]
    fn test_push_primitive() {
        let mut state = state!(
            globals => { "x" => 0 }
            locals => { "x" => 1 }
            stack => []
        );

        assert_eq!(PushPrimitive(2.into()).eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(state.stack_frames[0].stack[0], 2.into());
    }

    #[test]
    fn test_table_get() {
        let mut state = state!(
            stack => [
                table!["x" => 1],
                "x",
            ]
        );

        assert_eq!(TableGet.eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 2);
        assert_eq!(state.stack_frames[0].stack[1], 1.into());
    }

    #[test]
    fn test_table_list_build() {
        let mut state = state!(
            stack => [
                1,
                2,
                3,
            ]
        );

        assert_eq!(TableListBuild(3).eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().get_table(),
            Some(table![1, 2, 3])
        );
    }

    #[test]
    fn test_table_dict_build() {
        let mut state = state!(
            stack => [
                "a",
                1,
                "b",
                2,
                "c",
                3,
            ]
        );

        assert_eq!(TableDictBuild(3).eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().get_table(),
            Some(table!["a" => 1, "b" => 2, "c" => 3])
        );
    }

    #[test]
    fn test_table_merge() {
        let mut state = state!(
            stack => [
                table!["a" => 1, "b" => 2],
                table!["b" => 3, "c" => 4],
            ]
        );

        assert_eq!(TableMerge.eval(&mut state).unwrap(), false);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().get_table(),
            Some(table!["a" => 1, "b" => 3, "c" => 4])
        );
    }
}
