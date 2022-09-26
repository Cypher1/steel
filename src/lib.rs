// TODO: Remove when we can run in ECS and AST mode.

mod arena; // Boiler plate: should be a dependency.
pub mod ast;
mod compiler_context;
pub mod ecs;
mod error;
pub mod gen_code;
pub mod nodes;
mod parser;
mod interpreter;
mod pretty_printer;

#[cfg(test)]
#[macro_use]
mod assertions;
#[cfg(test)]
mod integration_tests;

pub use crate::compiler_context::CompilerContext;
use crate::interpreter::{EvalState, eval};
pub use crate::error::SteelErr;
use crate::parser::program;

pub fn handle<S: CompilerContext>(line: &str) -> Result<(), SteelErr>
where
    SteelErr: From<<S as CompilerContext>::E>,
{
    let mut store = S::new();
    let (_input, expr) = program(&mut store, line)?;
    eprintln!("expr: {:?}", store.pretty(expr));
    let mut state = EvalState::default();
    let result_index = state.setup_call(expr);
    eval(&store, &mut state)?;
    eprintln!("eval: {:?} {:?}", state, result_index);
    Ok(())
}

#[cfg(test)]
mod test {
use super::*;
use gen_code::{generate_random_program, Spec};

fn test_with_program<Ctx: CompilerContext>(){
    let size: usize = 100;
    let spec = Spec::default().sized(size);
    let mut rng = rand::thread_rng();
    let mut store = ast::Ast::new();
    let program = generate_random_program("ast generator", &mut store, &spec, &mut rng);
    let program = store.pretty(program);

    handle::<ecs::Ecs>(&program).expect(&format!("Should be able to eval program:\n{}", program))
}

#[test]
fn can_handle_random_programs_ast() {
    test_with_program::<ast::Ast>();
}

#[test]
fn can_handle_random_programs_ecs() {
    test_with_program::<ecs::Ecs>();
}

}
