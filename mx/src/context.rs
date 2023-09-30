use crate::Program;

pub struct Context {}

#[derive(Debug)]
pub enum CompileError {}

impl Context {
    pub fn new() -> Self {
        Context {}
    }

    /// Compile the given input into a program, without interning new constants.
    pub fn compile(&self, input: &str) -> Result<Program, CompileError> {
        Ok(Program::new())
    }

    /// Compile the given input into a program, possibly interning new constants.
    pub fn compile_mut(&mut self, input: &str) -> Result<Program, CompileError> {
        Ok(Program::new())
    }
}
