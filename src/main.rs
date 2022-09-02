// TODO: Remove when we can run in ECS and AST mode.
#![allow(unused)]

mod arena;
mod ast;
mod ecs;
mod error;
mod nodes;
mod parser;
mod primitives;

#[cfg(test)]
mod assertions;

use error::SteelErr;
use parser::{expr, hex_color, symbol_raw};

fn main() -> Result<(), SteelErr<'static>> {
    println!("Hello, world!");

    dbg!(hex_color("#2F14DF")?);
    dbg!(symbol_raw("hello   ")?);

    {
        let ref mut ctx = ast::Ast::new();
        dbg!(expr(ctx, "12+23")?);
    }
    {
        let ref mut ctx = ecs::ECS::new();
        dbg!(expr(ctx, "12+23")?);
    }
    Ok(())
}
