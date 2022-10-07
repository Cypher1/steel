// TODO: Remove when we can run in ECS and AST mode.

mod arena; // Boiler plate: should be a dependency.
pub mod ast;
mod compiler_context;
pub mod ecs;
mod error;
pub mod gen_code;
mod interpreter;
pub mod nodes;
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
pub struct Tasks<'a, ID> {
    program: GetProgram<'a, ID>,
    print: bool,
    eval: bool,
}

impl<'a, ID> Default for Tasks<'a, ID> {
    fn default() -> Self {
        Self {
            program: Nothing,
            print: false,
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
    pub fn and_print(self) -> Self {
        Self {
            print: true,
            ..self
        }
    }
    pub fn and_eval(self) -> Self {
        Self { eval: true, ..self }
    }
    pub fn all(program: &'a str) -> Self {
        Self::parse(program).and_print().and_eval()
    }
}

pub fn run<T: CompilerContext>(name: &str) {
    run_inner::<T>(name).expect("unexpected error");
}

fn run_inner<T: CompilerContext>(name: &str) -> Result<(), SteelErr> {
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
        let store = handle::<T>(Tasks::all(&line))?;
        debug!("{}: {:?}", name, store);
        println!("{:?}", store);
    }
}

pub fn handle<S: CompilerContext>(steps: Tasks<S::ID>) -> Result<(Option<S::ID>, i64), SteelErr> {
    let mut store = S::new();
    handle_steps(&mut store, steps)
}

pub fn handle_steps<S: CompilerContext>(
    store: &mut S,
    steps: Tasks<S::ID>,
) -> Result<(Option<S::ID>, i64), SteelErr> {
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
    if steps.eval {
        return Ok((Some(expr), eval_program(store, expr, &program_txt)?));
    }
    Ok((Some(expr), 0)) // TODO: Find a better result value
}

pub fn eval_program<S: CompilerContext>(
    store: &mut S,
    expr: S::ID,
    program_txt: &str,
) -> Result<i64, SteelErr> {
    let mut state = EvalState::default();
    let result_index = state.setup_eval(StaticPtr(expr), 0);
    eval(store, &mut state)?;
    let res = state.mem_stack.get(result_index);
    debug!("eval: {:?} {:?}", state, res);
    match res {
        Some(Value::I64(res)) => Ok(*res),
        Some(Value::Extern(_func)) => {
            panic!("Returned an extern func!? {:?}\n{}", res, program_txt)
        }
        Some(Value::UnInit) => panic!(
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

    fn test_with_random_program<Ctx: CompilerContext>(size: usize) {
        // TODO: use https://docs.rs/crate/quickcheck/0.9.2
        let spec = Spec::default().sized(size);
        let mut rng = rand::thread_rng();
        let mut store = ast::Ast::new();
        let program = generate_random_program("ast generator", &mut store, &spec, &mut rng);
        let program = store.pretty(program);
        take_result(
            &program,
            handle::<ecs::Ecs>(Tasks::parse(&program).and_eval()),
        )
    }
    const DEVIOUS_PROGRAM: &str = "0(putchar())";
    const SIMPLE_PROGRAM: &str = "putchar(48+9)";
    const MEDIUM_PROGRAM: &str = "putchar(65)+putchar(66)+putchar(67)+putchar(10)";

    #[test]
    #[should_panic]
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
        test_with_random_program::<ast::Ast>(100);
    }

    #[ignore]
    #[test]
    fn can_handle_most_random_programs_ast() {
        for i in 1..100 {
            eprintln!("ast: testing programs of size {:?}", i);
            for _run in 0..100 {
                test_with_random_program::<ast::Ast>(i);
            }
        }
    }

    #[test]
    #[should_panic]
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
        test_with_random_program::<ecs::Ecs>(100);
    }

    #[ignore]
    #[test]
    fn can_handle_most_random_programs_ecs() {
        for i in 1..100 {
            eprintln!("ecs: testing programs of size {:?}", i);
            for _run in 0..100 {
                test_with_random_program::<ecs::Ecs>(i);
            }
        }
    }
}
