use crate::parser::ParserContext;
use crate::error::SteelErr;
use crate::ecs::Ecs;
use crate::ast::Ast;
use crate::parser::expr;

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

fn run_test<'a, T: ParserContext<'a>>(name: &str, case: &Case<'a>, ref mut ctx: T) -> Result<(), SteelErr>
where
    <T as ParserContext<'a>>::ID: std::fmt::Debug,
    SteelErr: From<<T as ParserContext<'a>>::E>,
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

macro_rules! make_case {
    ($case: expr) => {
        Case::new($case)
    };
    ($case: expr, $mod: ident $(, $mods: ident)*) => {
        make_case!($case $(, $mods )*).$mod()
    };
}
macro_rules! make_test {
    ($name: ident, $case: expr $(, $mods: ident)*) => {
        #[test]
        fn $name() -> Result<(), SteelErr> {
            let case = make_case!($case $(, $mods)*);
            run_test("Ast", &case, Ast::new())?;
            run_test("Ecs", &case, Ecs::new())
        }
    };
}

make_test!(simple_plus, "(12+23)");
make_test!(unary_in_parens, "(*12)");
make_test!(unary_no_parens, "*12", no_round_trip);
make_test!(func_call, "foo(12, a)");
make_test!(op_call, "+(12, 23)");
make_test!(multi_op, "(12+23+34)", no_round_trip);
