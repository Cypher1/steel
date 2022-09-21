use crate::ast::Ast;
use crate::compiler_context::CompilerContext;
use crate::ecs::Ecs;
use crate::error::SteelErr;
use crate::parser::program;
use glasses::{glasses_harness, glasses_test};
use ntest::timeout;

#[derive(Default, Debug)]
pub struct Case<'a> {
    txt: Option<&'a str>,
    no_round_trip: bool,
    prints_as: Option<&'a str>,
    error_is: Option<&'a str>,
}

impl<'a> Case<'a> {
    fn expr(mut self, txt: &'a str) -> Self {
        self.txt = Some(txt);
        self
    }
    fn no_round_trip(mut self) -> Self {
        self.no_round_trip = true;
        self
    }
    fn error_is(mut self, error_is: &'a str) -> Self {
        self.error_is = Some(error_is);
        self
    }
    fn prints_as(mut self, prints_as: &'a str) -> Self {
        self.prints_as = Some(prints_as);
        self
    }
}

fn run_test<'a, T: CompilerContext<'a>>(
    name: &str,
    case: &Case<'a>,
    ref mut ctx: T,
) -> Result<(), SteelErr>
where
    <T as CompilerContext<'a>>::ID: std::fmt::Debug,
    SteelErr: From<<T as CompilerContext<'a>>::E>,
{
    let txt = case.txt.expect("Should have an input expression");
    eprintln!("TEST: {} -> {}", name, txt);
    let (left_over, result) = match program(ctx, txt) {
        Ok((left_over, result)) => (left_over, result),
        Err(e) => {
            let e = e.into();
            if let Some(error_is) = case.error_is {
                assert_eq!(error_is, format!("{}", e));
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };
    assert_eq!(left_over, "", "Expected to parse full input");
    eprint!("  program: {:?}", result);
    eprintln!(" value={:?}", ctx.get_call(result)?);
    let pretty = ctx.pretty(result);
    eprintln!(" as_tree={}", &pretty);
    if let Some(prints_as) = case.prints_as {
        assert_eq!(pretty, prints_as, "Is expected to print as");
    } else {
        if !case.no_round_trip {
            assert_eq!(pretty, txt, "Is expected to round trip");
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

glasses_harness!(ParserTest, Case<'static>, |case: Case<'static>| {
    run_test("Ast", &case, Ast::new()).expect("Ast failed");
    run_test("Ecs", &case, Ecs::new()).expect("Ast failed");
});

glasses_test!(ParserTest, handle_white_space, [timeout(10)], expr "-123\n", prints_as "(-123)");
glasses_test!(
    ParserTest,
    handle_malformed_with_white_space,
    [timeout(1)],
    expr "#lol\n",
    error_is "Expected an expression, found \"#lol\""
);
glasses_test!(ParserTest, simple_plus, [timeout(10)], expr "(12+23)");
glasses_test!(ParserTest, simple_plus_with_trailing, [timeout(10)], expr "(12+23)");
glasses_test!(ParserTest, unary_in_parens, [timeout(10)], expr "(*12)");
glasses_test!(ParserTest, unary_no_parens, [timeout(10)], expr "*12", no_round_trip);
glasses_test!(ParserTest, func_call, [timeout(10)], expr "foo(12, a)");
glasses_test!(ParserTest, op_call, [timeout(10)], expr "+(12, 23)");
glasses_test!(ParserTest, multi_op, [timeout(10)], expr "(12+23+34)", no_round_trip);
glasses_test!(ParserTest, prec_mul_add, [timeout(10)], expr "12*23+34", prints_as "((12*23)+34)");
glasses_test!(ParserTest, prec_add_mul, [timeout(10)], expr "12+23*34", prints_as "(12+(23*34))");
glasses_test!(
    ParserTest,
    prec_mul_paren_add,
    [timeout(10)],
    expr "12*(23+34)",
    prints_as "(12*(23+34))"
);
glasses_test!(
    ParserTest,
    prec_paren_add_mul,
    [timeout(10)],
    expr "(12+23)*34",
    prints_as "((12+23)*34)"
);
glasses_test!(ParserTest, prec_hard_case2, [timeout(10)], expr "a+b*c+d", prints_as "((a+(b*c))+d)");
