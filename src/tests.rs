use crate::ast::Ast;
use crate::ecs::Ecs;
use crate::error::SteelErr;
use crate::parser::program;
use crate::parser::ParserContext;
use ntest::timeout;

#[derive(Default, Debug)]
struct Case<'a> {
    txt: &'a str,
    no_round_trip: bool,
    prints_as: Option<&'a str>,
}

impl<'a> Case<'a> {
    fn new(txt: &'a str) -> Self {
        Case {
            txt,
            ..Default::default()
        }
    }

    fn no_round_trip(self) -> Self {
        Case {
            no_round_trip: true,
            ..self
        }
    }

    fn prints_as(self, prints_as: &'a str) -> Self {
        Case {
            prints_as: Some(prints_as),
            ..self
        }
    }
}

fn run_test<'a, T: ParserContext<'a>>(
    name: &str,
    case: &Case<'a>,
    ref mut ctx: T,
) -> Result<(), SteelErr>
where
    <T as ParserContext<'a>>::ID: std::fmt::Debug,
    SteelErr: From<<T as ParserContext<'a>>::E>,
{
    eprintln!("TEST: {} -> {}", name, case.txt);
    let (left_over, result) = program(ctx, case.txt)?;
    assert_eq!(left_over, "", "Expected to parse full input");
    eprint!("  program: {:?}", result);
    eprintln!(" value={:?}", ctx.get_call(result)?);
    let pretty = ctx.pretty(result);
    eprintln!(" as_tree={}", &pretty);
    if let Some(prints_as) = case.prints_as {
        assert_eq!(pretty, prints_as, "Is expected to print as");
    } else {
        if !case.no_round_trip {
            assert_eq!(pretty, case.txt, "Is expected to round trip");
        }
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
    ($case: expr, $mod: ident $($args: expr)* $(, $tail_mods: ident $($tail_args: expr)*)*) => {
        make_case!($case $(, $tail_mods $($tail_args)*)*).$mod($($args, )*)
    };
}
macro_rules! make_test {
    ($name: ident, $case: expr $(, $mods: ident $($args: expr)*)*) => {
        #[test]
        #[timeout(1)]
        fn $name() -> Result<(), SteelErr> {
            let case = make_case!($case $(, $mods $( $args )*)*);
            run_test("Ast", &case, Ast::new()).expect("Ast failed");
            run_test("Ecs", &case, Ecs::new()).expect("Ast failed");
            Ok(())
        }
    };
}

make_test!(handle_white_space, "-123\n", prints_as "(-123)");
make_test!(simple_plus, "(12+23)");
make_test!(simple_plus_with_trailing, "(12+23)");
make_test!(unary_in_parens, "(*12)");
make_test!(unary_no_parens, "*12", no_round_trip);
make_test!(func_call, "foo(12, a)");
make_test!(op_call, "+(12, 23)");
make_test!(multi_op, "(12+23+34)", no_round_trip);
make_test!(prec_mul_add, "12*23+34", prints_as "((12*23)+34)");
make_test!(prec_add_mul, "12+23*34", prints_as "(12+(23*34))");
make_test!(prec_mul_paren_add, "12*(23+34)", prints_as "(12*(23+34))");
make_test!(prec_paren_add_mul, "(12+23)*34", prints_as "((12+23)*34)");
make_test!(prec_hard_case2, "a+b*c+d", prints_as "((a+(b*c))+d)");
