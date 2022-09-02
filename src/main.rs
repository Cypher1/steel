// TODO: Remove when we can run in ECS and AST mode.

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
use parser::expr;

fn test<'a, T: ParserContext<'a>>(name: &str, ref mut ctx: T) -> Result<(), SteelErr<'a>>
where
    <T as ParserContext<'a>>::ID: std::fmt::Debug,
{
    eprintln!("{} expr: {:?}", name, expr(ctx, "(12+23)")?);
    eprintln!(
        "    Mem usage: {:?}/{:?}",
        ctx.active_mem_usage(),
        ctx.mem_usage()
    );
    eprintln!("    Base structure: {:?}", std::mem::size_of::<T>());
    Ok(())
}

fn main() -> Result<(), SteelErr<'static>> {
    test("Ast", ast::Ast::new())?;
    test("Ecs", ecs::Ecs::new())?;
    Ok(())
}
