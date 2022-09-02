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

use crate::parser::ParserContext;
use error::SteelErr;
use parser::{expr, hex_color, symbol_raw};

fn test<'a, T: ParserContext<'a>>(name: &str, ref mut ctx: T) -> Result<(), SteelErr<'a>>
where
    <T as ParserContext<'a>>::ID: std::fmt::Debug,
{
    eprintln!("{} expr: {:?}", name, expr(ctx, "12+23")?);
    eprintln!(
        "Mem usage: {:?}/{:?}",
        ctx.active_mem_usage(),
        ctx.mem_usage()
    );
    eprintln!("Base structure: {:?}", std::mem::size_of::<T>());
    Ok(())
}

fn main() -> Result<(), SteelErr<'static>> {
    println!("Hello, world!");

    eprintln!("Parse hex: {:?}", hex_color("#2F14DF")?);
    eprintln!("Parse symbol: {:?}", symbol_raw("hello   ")?);

    test("Ast", ast::Ast::new());
    test("Ecs", ecs::Ecs::new());
    Ok(())
}
