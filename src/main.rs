// TODO: Remove when we can run in ECS and AST mode.

mod arena; // Boiler plate: should be a dependency.
mod ast;
mod compiler_context;
mod ecs;
mod error;
mod nodes;
mod parser;

#[cfg(test)]
#[macro_use]
mod assertions;
#[cfg(test)]
mod tests;

use compiler_context::CompilerContext;
use error::SteelErr;
use parser::program;

use crate::compiler_context::EvalState;

fn handle<'a, S: CompilerContext<'a>>(line: &'a str) -> Result<(), SteelErr>
where
    SteelErr: From<<S as CompilerContext<'a>>::E>,
{
    let mut store = S::new();
    let (_input, program) = program(&mut store, line)?;
    eprintln!("expr: {:?}", store.pretty(program));
    let mut stack = EvalState::default();
    stack.function_stack.push(program);
    store.eval(&mut stack)?;
    eprintln!("eval: {:?}", stack);
    Ok(())
}

fn main() -> Result<(), SteelErr> {
    let mut args = std::env::args();
    let _program_path = args.next();
    for arg in args {
        eprintln!("unknown argument: {}", arg);
        std::process::exit(1);
    }
    loop {
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line)? == 0 {
            return Ok(());
        }
        eprintln!("line: {}", line);
        handle::<ast::Ast>(&line)?;
        handle::<ecs::Ecs>(&line)?;
    }
}
