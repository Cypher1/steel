// TODO: Remove when we can run in ECS and AST mode.

mod arena;
mod ast;
mod ecs;
mod error;
mod nodes;
mod parser;

#[cfg(test)]
#[macro_use]
mod assertions;
#[cfg(test)]
mod tests;

use crate::parser::ParserContext;
use error::SteelErr;
use parser::program;

fn handle<'a, S: ParserContext<'a>>(line: &'a str)
where
    S::E: Into<SteelErr>,
{
    let mut store = S::new();
    match program(&mut store, line) {
        Ok((_input, program)) => {
            eprintln!("ast expr: {:?}", store.pretty(program));
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
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
        handle::<ast::Ast>(&line);
        handle::<ecs::Ecs>(&line);
    }
}
