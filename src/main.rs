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

#[derive(Default, Debug)]
struct Case<'a> {
    txt: &'a str,
    no_round_trip: bool,
}

impl<'a> Case<'a> {
    fn new(txt: &'a str) -> Self {
        Case {
            txt,
            ..Default::default()
        }
    }

    fn no_round_trip(self) -> Self {
        Case { no_round_trip: true, ..self }
    }
}

fn test<'a, T: ParserContext<'a>>(name: &str, case: &Case<'a>, ref mut ctx: T) -> Result<(), SteelErr<'a>>
where
    <T as ParserContext<'a>>::ID: std::fmt::Debug,
    SteelErr<'a>: From<<T as ParserContext<'a>>::E>,
{
    eprintln!("TEST: {} -> {}", name, case.txt);
    let (left_over, result) = expr(ctx, case.txt)?;
    assert_eq!(left_over, "", "Is expected to fully parse the input");
    eprint!("  expr: {:?}", result);
    eprintln!(" value={:?}", ctx.get_call(result)?);
    let pretty = ctx.pretty(result);
    eprintln!(" as_tree={}", &pretty);
    if !case.no_round_trip {
        assert_eq!(pretty, case.txt, "Is expected to round trip");
    }
    eprintln!(
        "    Mem usage: {:?}/{:?}",
        ctx.active_mem_usage(),
        ctx.mem_usage()
    );
    eprintln!("    Base structure: {:?}", std::mem::size_of::<T>());
    Ok(())
}

fn main() -> Result<(), SteelErr<'static>> {
    let cases = [
        Case::new("(12+23)"),
        Case::new("(*12)"),
        Case::new("*12").no_round_trip(),
        Case::new("foo(12, a)"),
        Case::new("+(12, 23)"),
        Case::new("(12+23+34)"),
    ];
    for case in cases {
        test("Ast", &case, ast::Ast::new())?;
        test("Ecs", &case, ecs::Ecs::new())?;
    }
    Ok(())
}
