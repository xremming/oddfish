use std::collections::HashMap;

use crate::{
    ops::{BinaryOp, UnaryOp},
    Primitive, Table, Value,
};

#[derive(Debug)]
pub enum RuntimeError {
    EmptyStack,
    NoStackFrames,
    NotATable,
    InvalidVariable,
    InvalidTableKey,
    InvalidProgramCounter,
    InvalidReturnAddress,
    InvalidCallable,
    FunctionArgumentNotProvided,
}

type Vars = HashMap<String, Value>;

struct StackFrame {
    locals: Vars,
    return_address: Option<usize>,
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

    fn push_frame(&mut self, locals: Vars, return_address: usize) {
        self.stack_frames.push(StackFrame {
            locals,
            return_address: Some(return_address),
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
    Swap,
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

    /// Push the value at the given name onto the stack.
    ///
    /// `TOS = var[TOS]`
    PushName,
    /// Push the given constant onto the stack.
    PushPrimitive(Primitive),

    /// Get value from table at TOS1 with key TOS and push it onto the stack.
    ///
    /// `TOS = TOS1[TOS]`
    TableGet,
    TableListBuild(usize),
    TableDictBuild(usize),
    TableMerge,

    /// if TOS is true, jump forwards by the given amount
    PopJumpIfTrue(usize),
    /// if TOS is false, jump forwards by the given amount
    PopJumpIfFalse(usize),
    /// Jump forwards by the given amount. `Jump(0)` is considered a `Nop`.
    Jump(usize),

    PushFunction(usize),
    /// Pops N key pairs from the stack, then gets those values from the table at the TOS.
    /// First value of the pair is tried first (named argument) and the second value is used as
    /// only if the first one is not found (positional argument).
    ///
    /// The first value MUST BE a String.
    ///
    /// The second value MUST BE a Number or Nil, with Nil meaning not to get a second value.
    ///
    /// That is, it does more or less the following pseudocode.
    /// ```pseudo-code
    /// // f => (self, a, b, c) => ...
    /// f => (...args) => {
    ///   self = args["self"] ?? args[nil]
    ///   a = args["a"] ?? args[0]
    ///   b = args["b"] ?? args[1]
    ///   c = args["c"] ?? args[2]
    /// }
    /// ```
    StoreFunctionArgs(bool, usize),
    /// TOS = TOS(TOS1)
    Call,
    /// return TOS
    Return,
}

impl Instruction {
    fn eval(&self, state: &mut State) -> Result<Option<Value>, RuntimeError> {
        debug_assert!(state.stack_frames.len() > 0);

        use Instruction::*;
        match self {
            Nop => {
                state.incr_pc();
                Ok(None)
            }
            Copy => {
                state.incr_pc();
                let value = state.peek_stack()?.clone();
                state.push_stack(value)?;
                Ok(None)
            }
            Swap => {
                state.incr_pc();
                let tos = state.pop_stack()?;
                let tos1 = state.pop_stack()?;
                state.push_stack(tos)?;
                state.push_stack(tos1)?;
                Ok(None)
            }
            Pop => {
                state.incr_pc();
                state.pop_stack()?;
                Ok(None)
            }
            StoreName => {
                state.incr_pc();
                let value = state.pop_stack()?;
                let name = state.pop_stack()?;
                let var = name
                    .get_value::<String>()
                    .ok_or(RuntimeError::InvalidVariable)?;
                state.set_var(&var, value)?;
                Ok(None)
            }
            StorePrimitive(value) => {
                state.incr_pc();
                let name = state.pop_stack()?;
                let var = name
                    .get_value::<String>()
                    .ok_or(RuntimeError::InvalidVariable)?;
                state.set_var(&var, value.clone().into())?;
                Ok(None)
            }
            PushName => {
                state.incr_pc();
                let name = state.pop_stack()?;
                let var = name
                    .get_value::<String>()
                    .ok_or(RuntimeError::InvalidVariable)?;
                let value = state.get_var(&var)?;
                state.push_stack(value)?;
                Ok(None)
            }
            PushPrimitive(value) => {
                state.incr_pc();
                state.push_stack(value.clone().into())?;
                Ok(None)
            }
            TableGet => {
                state.incr_pc();
                let key = state
                    .pop_stack()?
                    .into_primitive()
                    .ok_or(RuntimeError::InvalidTableKey)?;
                let tos = state.peek_stack()?;
                match tos {
                    Value::Table(table) => {
                        let value = table.get(key).cloned().unwrap_or(Value::nil());
                        state.push_stack(value)?;
                        Ok(None)
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
                Ok(None)
            }
            TableDictBuild(n) => {
                state.incr_pc();
                let mut table = Table::new();

                let key_value_pairs = {
                    let mut res = Vec::with_capacity(*n * 2);
                    for _ in 0..*n {
                        let value = state.pop_stack()?;
                        let key = state
                            .pop_stack()?
                            .into_primitive()
                            .ok_or(RuntimeError::InvalidTableKey)?;
                        res.push((key, value));
                    }
                    res
                };

                for (key, value) in key_value_pairs.into_iter().rev() {
                    table.set(key, value);
                }

                state.push_stack(table.into())?;

                Ok(None)
            }
            TableMerge => {
                state.incr_pc();
                let tos: Table = state
                    .pop_stack()?
                    .into_table()
                    .ok_or(RuntimeError::NotATable)?;
                let mut tos1: Table = state
                    .pop_stack()?
                    .into_table()
                    .ok_or(RuntimeError::NotATable)?;

                for (key, value) in tos {
                    tos1.set(key, value);
                }

                state.push_stack(tos1.into())?;
                Ok(None)
            }
            StoreFunctionArgs(get_self, n) => {
                state.incr_pc();

                let positional_named_pairs = {
                    let mut res = Vec::with_capacity(*n * 2);
                    for _ in 0..*n {
                        let positional = state.pop_stack()?;
                        let named = state.pop_stack()?;
                        res.push((positional, named));
                    }
                    res
                };

                let mut tos = state
                    .pop_stack()?
                    .into_table()
                    .ok_or(RuntimeError::InvalidVariable)?;
                for (positional, named) in positional_named_pairs {
                    let named = named
                        .get_value::<String>()
                        .ok_or(RuntimeError::InvalidVariable)?;
                    let mut value = tos.get_mut(named.clone());
                    if value.is_nil() {
                        let positional = positional
                            .get_value::<f64>()
                            .ok_or(RuntimeError::InvalidVariable)?;
                        value = tos.get_mut(positional);
                        if value.is_nil() {
                            return Err(RuntimeError::FunctionArgumentNotProvided);
                        }
                    }

                    state.set_var(&named, value.clone())?;
                    *value = ().into();
                }

                if *get_self {
                    let self_value = tos.get_mut("self");
                    if self_value.is_nil() {
                        return Err(RuntimeError::FunctionArgumentNotProvided);
                    }
                    state.set_var("self", self_value.clone())?;
                    *self_value = ().into();
                }

                Ok(None)
            }
            Return => {
                let value = state.pop_stack()?;
                let frame = state
                    .stack_frames
                    .pop()
                    .ok_or(RuntimeError::NoStackFrames)?;

                // in general, only the first stack frame should not have a
                // return address, either way returning will either jump to
                // the return address or stop the program
                if let Some(return_address) = frame.return_address {
                    state.pc = return_address;
                    state.push_stack(value)?;
                    Ok(None)
                } else {
                    Ok(Some(value))
                }
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

    fn run_with_state(&self, mut state: &mut State) -> Result<Value, RuntimeError> {
        loop {
            if let Some(instruction) = self.instructions.get(state.pc) {
                if let Some(return_value) = instruction.eval(&mut state)? {
                    return Ok(return_value);
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

    pub fn run(&self) -> Result<Value, RuntimeError> {
        self.run_with(HashMap::new())
    }

    pub fn run_with(&self, globals: Vars) -> Result<Value, RuntimeError> {
        let mut state = State {
            pc: 0,
            globals,
            stack_frames: vec![StackFrame {
                locals: HashMap::new(),
                return_address: None,
                stack: Vec::new(),
            }],
        };

        self.run_with_state(&mut state)
    }

    pub fn call(&self, value: Value, args: Vars) -> Result<Value, RuntimeError> {
        self.call_with(value, args, HashMap::new())
    }

    pub fn call_with(
        &self,
        value: Value,
        args: Vars,
        globals: Vars,
    ) -> Result<Value, RuntimeError> {
        match value {
            Value::FunctionPointer(function) => {
                let mut state = State {
                    pc: function,
                    globals,
                    stack_frames: vec![StackFrame {
                        locals: args,
                        return_address: None,
                        stack: Vec::new(),
                    }],
                };
                self.run_with_state(&mut state)
            }
            _ => Err(RuntimeError::InvalidCallable),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Instruction::*;
    use super::*;
    use crate::{
        ops::{BinaryOp, UnaryOp},
        table,
    };

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
                    return_address: None,
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
        assert_eq!(Nop.eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
    }

    #[test]
    fn test_copy() {
        let mut state = state!(
            stack => [1]
        );

        assert_eq!(Copy.eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().into_primitive(),
            Some(1.into())
        );
        assert_eq!(
            state.stack_frames[0].stack[1].clone().into_primitive(),
            Some(1.into())
        );
    }

    #[test]
    fn test_swap() {
        let mut state = state!(
            stack => [1, 2]
        );

        assert_eq!(Swap.eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().into_primitive(),
            Some(2.into())
        );
        assert_eq!(
            state.stack_frames[0].stack[1].clone().into_primitive(),
            Some(1.into())
        );
    }

    #[test]
    fn test_pop() {
        let mut state = state!(
            stack => [1]
        );

        assert_eq!(Pop.eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 0);
    }

    #[test]
    fn test_store_name() {
        let mut state = state!(
            globals => { "x" => 0 }
            locals => { "x" => 1 }
            stack => ["x", 2]
        );

        assert_eq!(StoreName.eval(&mut state).unwrap(), None);
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

        assert_eq!(StorePrimitive(2.into()).eval(&mut state).unwrap(), None);
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

        assert_eq!(PushName.eval(&mut state).unwrap(), None);
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

        assert_eq!(PushPrimitive(2.into()).eval(&mut state).unwrap(), None);
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

        assert_eq!(TableGet.eval(&mut state).unwrap(), None);
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

        assert_eq!(TableListBuild(3).eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().into_table(),
            Some(table![1, 2, 3])
        );
    }

    #[test]
    fn test_table_dict_build() {
        let mut state = state!(
            stack => [
                "a",
                2,
                "a",
                1,
                "b",
                2,
                "c",
                3,
            ]
        );

        assert_eq!(TableDictBuild(4).eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().into_table(),
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

        assert_eq!(TableMerge.eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].stack.len(), 1);
        assert_eq!(
            state.stack_frames[0].stack[0].clone().into_table(),
            Some(table!["a" => 1, "b" => 3, "c" => 4])
        );
    }

    #[test]
    fn test_store_function_args() {
        let mut state = state!(
            locals => {
                "self" => 0,
                "a" => 0,
                "b" => 0,
                "c" => 0,
            }
            stack => [
                table!["self" => 1, 0 => 2, "b" => 3, 2 => 4],
                "a",
                0,
                "b",
                (),
                "c",
                2,
            ]
        );

        assert_eq!(StoreFunctionArgs(true, 3).eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].locals.len(), 4);
        assert_eq!(state.stack_frames[0].locals.get("self").unwrap(), &1.into());
        assert_eq!(state.stack_frames[0].locals.get("a").unwrap(), &2.into());
        assert_eq!(state.stack_frames[0].locals.get("b").unwrap(), &3.into());
        assert_eq!(state.stack_frames[0].locals.get("c").unwrap(), &4.into());
    }

    #[test]
    fn test_store_function_args_named_no_self() {
        let mut state = state!(
            locals => {
                "self" => 0,
                "a" => 0,
                "b" => 0,
                "c" => 0,
            }
            stack => [
                table!["a" => 1, "b" => 2, "c" => 3],
                "a",
                (),
                "b",
                (),
                "c",
                (),
            ]
        );

        assert_eq!(StoreFunctionArgs(false, 3).eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].locals.len(), 4);
        assert_eq!(state.stack_frames[0].locals.get("self").unwrap(), &0.into());
        assert_eq!(state.stack_frames[0].locals.get("a").unwrap(), &1.into());
        assert_eq!(state.stack_frames[0].locals.get("b").unwrap(), &2.into());
        assert_eq!(state.stack_frames[0].locals.get("c").unwrap(), &3.into());
    }

    #[test]
    fn test_store_function_args_positional_no_self() {
        let mut state = state!(
            locals => {
                "self" => 0,
                "a" => 0,
                "b" => 0,
                "c" => 0,
            }
            stack => [
                table![0 => 1, 1 => 2, 2 => 3],
                "a",
                0,
                "b",
                1,
                "c",
                2,
            ]
        );

        assert_eq!(StoreFunctionArgs(false, 3).eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].locals.len(), 4);
        assert_eq!(state.stack_frames[0].locals.get("self").unwrap(), &0.into());
        assert_eq!(state.stack_frames[0].locals.get("a").unwrap(), &1.into());
        assert_eq!(state.stack_frames[0].locals.get("b").unwrap(), &2.into());
        assert_eq!(state.stack_frames[0].locals.get("c").unwrap(), &3.into());
    }

    #[test]
    fn test_store_function_args_with_self() {
        let mut state = state!(
            locals => {
                "self" => 0,
                "a" => 0,
                "b" => 0,
                "c" => 0,
            }
            stack => [
                table!["self" => 1, 0 => 2, "b" => 3, "c" => 4],
                "a",
                0,
                "b",
                (),
                "c",
                (),
            ]
        );

        assert_eq!(StoreFunctionArgs(true, 3).eval(&mut state).unwrap(), None);
        assert_eq!(state.pc, 1);
        assert_eq!(state.stack_frames[0].locals.len(), 4);
        assert_eq!(state.stack_frames[0].locals.get("self").unwrap(), &1.into());
        assert_eq!(state.stack_frames[0].locals.get("a").unwrap(), &2.into());
        assert_eq!(state.stack_frames[0].locals.get("b").unwrap(), &3.into());
        assert_eq!(state.stack_frames[0].locals.get("c").unwrap(), &4.into());
    }

    macro_rules! program {
        ($($v:expr),* $(,)?) => {
            {
                #[allow(unused_mut)]
                let mut program = Program::new();
                $(
                    program.instructions.push($v);
                )*
                program
            }
        };
    }

    #[test]
    fn test_empty_program() {
        let program = program![];
        let res = program.run().unwrap();
        assert_eq!(res, Value::nil());
    }

    #[test]
    #[rustfmt::skip]
    fn test_return_one() {
        // return 1
        let program = program![
            PushPrimitive(1.into()),
            Return,
        ];

        let res = program.run().unwrap();
        assert_eq!(res, 1.into());
    }

    #[ignore = "UnaryOp is not yet implemented"]
    #[test]
    #[rustfmt::skip]
    fn test_minus_one() {
        // return -1
        let program = program![
            PushPrimitive(1.into()),
            UnaryOp(UnaryOp::Minus),
            Return,
        ];

        let res = program.run().unwrap();
        assert_eq!(res, (-1).into());
    }

    #[ignore = "BinaryOp is not yet implemented"]
    #[test]
    fn test_one_plus_one() {
        // return 1 + 1
        let program = program![
            PushPrimitive(1.into()),
            PushPrimitive(1.into()),
            BinaryOp(BinaryOp::Add),
            Return,
        ];

        let res = program.run().unwrap();
        assert_eq!(res, 2.into());
    }
}
