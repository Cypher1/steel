use steel::{ast, ecs, handle, SteelErr};
use log::{debug, error};

fn main() -> Result<(), SteelErr> {
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
        debug!("ast: {:?}", handle::<ast::Ast>(&line)?);
        debug!("ecs: {:?}", handle::<ecs::Ecs>(&line)?);
    }
}
