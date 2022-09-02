// TODO: Remove when we can run in ECS and AST mode.

mod arena;
mod ast;
mod ecs;
mod error;
mod nodes;
mod parser;

#[cfg(test)]
mod assertions;
#[cfg(test)]
mod tests;

use crate::parser::ParserContext;
use error::SteelErr;
use parser::program;

fn main() -> Result<(), SteelErr> {
    let args = std::env::args();
    for arg in args {
        println!("arg: {}", arg);
    }
    for line in std::io::stdin().lines() {
        let line = line.expect("Couldn't read from stdin");
        println!("line: {}", line);
        {
            let line = line.clone();
            let mut ast = ast::Ast::new();
            let (_ast_out_input, ast_out) = program(&mut ast, &line)?;
            println!("ast expr: {:?}", ast.pretty(ast_out));
        }
        {
            let line = line.clone();
            let mut ecs = ecs::Ecs::new();
            let (_ecs_out_input, ecs_out) = program(&mut ecs, &line)?;
            println!("ecs expr: {:?}", ecs.pretty(ecs_out));
        }
    }
    Ok(())
}
