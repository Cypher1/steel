// TODO: Remove when we can run in ECS and AST mode.

mod arena; // Boiler plate: should be a dependency.
pub mod ast;
mod compiler_context;
pub mod ecs;
mod error;
pub mod gen_code;
mod interpreter;
pub mod nodes;
mod optimizer;
mod parser;
mod pretty_printer;

#[cfg(test)]
#[macro_use]
mod assertions;
#[cfg(test)]
mod integration_tests;

pub use crate::compiler_context::CompilerContext;
pub use crate::error::SteelErr;
use crate::interpreter::{eval, EvalState, StaticPtr, Value};
use crate::parser::program;
use log::{debug, error};

#[derive(Debug, Default)]
pub enum GetProgram<'a, ID> {
    #[default]
    Nothing,
    FromStr(&'a str),
    FromStore(ID),
}
use GetProgram::*;

#[derive(Debug)]
#[non_exhaustive]
pub struct Tasks<'a, ID> {
    program: GetProgram<'a, ID>,
    print: bool,
    optimize: optimizer::Optimizations,
    print_optimized: bool,
    eval: bool,
}

impl<'a, ID> Default for Tasks<'a, ID> {
    fn default() -> Self {
        Self {
            program: Nothing,
            print: false,
            optimize: optimizer::Optimizations::none(),
            print_optimized: false,
            eval: false,
        }
    }
}

impl<'a, ID> Tasks<'a, ID> {
    pub fn parse(program: &'a str) -> Self {
        Self {
            program: FromStr(program),
            ..Self::default()
        }
    }
    pub fn pre_parsed(program: ID) -> Self {
        Self {
            program: FromStore(program),
            ..Self::default()
        }
    }
    pub fn and_optimize(self) -> Self {
        Self {
            optimize: self.optimize.all(),
            ..self
        }
    }
    pub fn and_print(self) -> Self {
        Self {
            print: true,
            ..self
        }
    }
    pub fn and_print_optimized(self) -> Self {
        Self {
            print_optimized: true,
            ..self
        }
    }
    pub fn and_eval(self) -> Self {
        Self { eval: true, ..self }
    }
    pub fn all(program: &'a str) -> Self {
        Self::parse(program)
            .and_print()
            .and_optimize()
            .and_print_optimized()
            .and_eval()
    }
}

pub fn run<Ctx: CompilerContext>(name: &str)
where
    SteelErr: From<<Ctx as CompilerContext>::E>,
{
    run_inner::<Ctx>(name).expect("unexpected error");
}

fn run_inner<Ctx: CompilerContext>(name: &str) -> Result<(), SteelErr>
where
    SteelErr: From<<Ctx as CompilerContext>::E>,
{
    env_logger::init();
    let mut args = std::env::args();
    let _program_path = args.next();
    for arg in args {
        error!("unknown argument: {}", arg);
        std::process::exit(1);
    }
    loop {
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line)? == 0 {
            return Ok(());
        }
        debug!("line: {}", line);
        let store = handle::<Ctx>(Tasks::all(&line))?;
        debug!("{}: {:?}", name, store);
        println!("{:?}", store);
    }
}

pub fn handle<Ctx: CompilerContext>(
    steps: Tasks<Ctx::ID>,
) -> Result<(Option<Ctx::ID>, i64), SteelErr>
where
    SteelErr: From<<Ctx as CompilerContext>::E>,
{
    let mut store = Ctx::new();
    handle_steps(&mut store, steps)
}

pub fn handle_steps<Ctx: CompilerContext>(
    store: &mut Ctx,
    steps: Tasks<Ctx::ID>,
) -> Result<(Option<Ctx::ID>, i64), SteelErr>
where
    SteelErr: From<<Ctx as CompilerContext>::E>,
{
    let (program_txt, expr) = match steps.program {
        Nothing => return Ok((None, 0)),
        FromStr(program_txt) => {
            let (_input, expr) = program(store, program_txt)?;
            (program_txt.to_string(), expr)
        }
        FromStore(expr) => (store.pretty(expr), expr),
    };
    debug!("expr: {:?}", store.pretty(expr));
    if steps.print {
        eprintln!(" {:?}", store.pretty(expr));
    }
    let expr = if steps.optimize == optimizer::Optimizations::none() {
        store.optimize(&steps.optimize, expr)?
    } else {
        expr
    };
    if steps.print_optimized {
        eprintln!(" {:?}", store.pretty(expr));
    }
    if steps.eval {
        return Ok((Some(expr), eval_program(store, expr, &program_txt)?));
    }
    Ok((Some(expr), 0)) // TODO: Find a better result value
}

pub fn eval_program<Ctx: CompilerContext>(
    store: &mut Ctx,
    expr: Ctx::ID,
    program_txt: &str,
) -> Result<i64, SteelErr> {
    let mut state = EvalState::default();
    let result_index = state.setup_eval(StaticPtr(expr), Vec::new());
    eval(store, &mut state)?;
    let res = state.mem_stack.get(result_index);
    debug!("eval: {:?} {:?}", state, res);
    match res {
        Some(Value::I64(res)) => Ok(*res),
        Some(Value::Extern(_func)) => {
            panic!("Returned an extern func!? {:?}\n{}", res, program_txt)
        }
        Some(Value::Uninit) => panic!(
            "No value was placed in the return address!?\n{}",
            program_txt
        ),
        None => panic!("The return address is out of bounds!?\n{}", program_txt),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gen_code::{generate_random_program, Spec};

    fn take_result<T: std::fmt::Debug, E: std::fmt::Debug>(program: &str, res: Result<T, E>) {
        // TODO: Should happen in an init phase.
        let _ = env_logger::builder().is_test(true).try_init();
        match res {
            Ok(r) => debug!("result {:?}", r),
            Err(e) => {
                eprintln!("Should be able to eval program:");
                eprintln!("{}", program);
                panic!("error: {:?}", e);
            }
        }
    }

    fn test_with_random_program<Ctx: CompilerContext>(size: usize) -> String {
        // TODO: use https://docs.rs/crate/quickcheck/0.9.2
        let spec = Spec::default().sized(size);
        let mut rng = rand::thread_rng();
        let mut store = ast::Ast::new();
        let program = generate_random_program("ast generator", &mut store, &spec, &mut rng);
        let program = store.pretty(program);
        take_result(
            &program,
            handle::<ecs::Ecs>(Tasks::parse(&program).and_eval()),
        );
        program
    }

    fn test_random_programs<Ctx: CompilerContext>(name: &str, max_size: usize, runs: usize) {
        for i in 1..max_size {
            eprintln!("{}: testing programs of size {:?}", name, i);
            for _run in 0..runs {
                test_with_random_program::<Ctx>(i);
            }
        }
    }

    const DEVIOUS_PROGRAM: &str = "0(putchar())";
    const SIMPLE_PROGRAM: &str = "putchar(48+9)";
    const MEDIUM_PROGRAM: &str = "putchar(65)+putchar(66)+putchar(67)+putchar(10)";

    #[test]
    fn cannot_handle_devious_program_ast() {
        let program = DEVIOUS_PROGRAM;
        take_result(program, handle::<ast::Ast>(Tasks::all(program)))
    }

    #[test]
    fn can_handle_simple_program_ast() {
        let program = SIMPLE_PROGRAM;
        take_result(program, handle::<ast::Ast>(Tasks::all(program)))
    }

    #[test]
    fn can_handle_medium_program_ast() {
        let program = MEDIUM_PROGRAM;
        take_result(program, handle::<ast::Ast>(Tasks::all(program)))
    }

    #[test]
    fn can_handle_random_programs_ast() {
        eprintln!(
            "sample program: {}",
            test_with_random_program::<ast::Ast>(1000)
        );
    }

    #[test]
    fn can_handle_small_random_programs_ast() {
        test_random_programs::<ast::Ast>("ast", 50, 100);
    }

    #[ignore]
    #[test]
    fn can_handle_most_random_programs_ast() {
        test_random_programs::<ast::Ast>("ast", 200, 1000);
    }

    #[test]
    fn cannot_handle_devious_program_ecs() {
        let program = DEVIOUS_PROGRAM;
        take_result(program, handle::<ecs::Ecs>(Tasks::all(program)))
    }

    #[test]
    fn can_handle_simple_program_ecs() {
        let program = SIMPLE_PROGRAM;
        take_result(program, handle::<ecs::Ecs>(Tasks::all(program)))
    }

    #[test]
    fn can_handle_medium_program_ecs() {
        let program = MEDIUM_PROGRAM;
        take_result(program, handle::<ecs::Ecs>(Tasks::all(program)))
    }

    #[test]
    fn can_handle_random_programs_ecs() {
        eprintln!(
            "sample program: {}",
            test_with_random_program::<ecs::Ecs>(1000)
        );
    }

    #[test]
    fn can_handle_small_random_programs_ecs() {
        test_random_programs::<ecs::Ecs>("ecs", 50, 100);
    }

    #[ignore]
    #[test]
    fn can_handle_most_random_programs_ecs() {
        test_random_programs::<ecs::Ecs>("ecs", 200, 1000);
    }
}
