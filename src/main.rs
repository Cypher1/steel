// TODO: Remove when we can run in ECS and AST mode.

mod arena;
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

fn handle<'a, S: CompilerContext<'a>>(line: &'a str) -> Result<(), SteelErr>
where
    S::E: Into<SteelErr>,
{
    let mut store = S::new();
    let (_input, program) = program(&mut store, line)?;
    eprintln!("expr: {:?}", store.pretty(program));
    Ok(())
}

fn main() -> Result<(), SteelErr> {
    let args = std::env::args();
    for arg in args {
        eprintln!("arg: {}", arg);
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
