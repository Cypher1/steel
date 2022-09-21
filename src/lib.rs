// TODO: Remove when we can run in ECS and AST mode.

mod arena; // Boiler plate: should be a dependency.
pub mod ast;
mod compiler_context;
pub mod ecs;
mod error;
mod nodes;
mod parser;

#[cfg(test)]
#[macro_use]
mod assertions;
#[cfg(test)]
mod tests;

use crate::compiler_context::CompilerContext;
use crate::compiler_context::EvalState;
pub use crate::error::SteelErr;
use crate::parser::program;

pub fn handle<'a, S: CompilerContext<'a>>(line: &'a str) -> Result<(), SteelErr>
where
    SteelErr: From<<S as CompilerContext<'a>>::E>,
{
    let mut store = S::new();
    let (_input, program) = program(&mut store, line)?;
    // eprintln!("expr: {:?}", store.pretty(program));
    let mut stack = EvalState::default();
    stack.function_stack.push(program);
    store.eval(&mut stack)?;
    // eprintln!("eval: {:?}", stack);
    Ok(())
}
