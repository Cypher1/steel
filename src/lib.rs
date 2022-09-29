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
use crate::interpreter::{eval, EvalState};
use crate::parser::program;
use log::{debug, error};

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
        let store = handle::<T>(&line)?;
        debug!("{}: {:?}", name, store);
        println!("{:?}", store);
    }
}

pub fn handle<S: CompilerContext>(line: &str) -> Result<i64, SteelErr> {
    let mut store = S::new();
    let (_input, expr) = program(&mut store, line)?;
    debug!("expr: {:?}", store.pretty(expr));
    let mut state = EvalState::default();
    let result_index = state.setup_call(expr, 0);
    eval(&store, &mut state).map_err(Into::into)?;
    debug!("eval: {:?} {:?}", state, state.mem_stack.get(result_index));
    Ok(state.mem_stack[result_index])
}

#[cfg(test)]
mod test {
    use super::*;
    use gen_code::{generate_random_program, Spec};
    use log::error;

    fn test_with_program<Ctx: CompilerContext>() {
        let size: usize = 100;
        let spec = Spec::default().sized(size);
        let mut rng = rand::thread_rng();
        let mut store = ast::Ast::new();
        let program = generate_random_program("ast generator", &mut store, &spec, &mut rng);
        let program = store.pretty(program);

        match handle::<ecs::Ecs>(&program) {
            Ok(r) => debug!("result {:?}", r),
            Err(e) => {
                error!("Should be able to eval program:");
                error!("{}", program);
                error!("error: {:?}", e);
            }
        }
    }

    #[test]
    fn can_handle_random_programs_ast() {
        test_with_program::<ast::Ast>();
    }

    #[test]
    fn can_handle_random_programs_ecs() {
        test_with_program::<ecs::Ecs>();
    }
}
