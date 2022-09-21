use steel::{ast, ecs, handle, SteelErr};

fn main() -> Result<(), SteelErr> {
    let mut args = std::env::args();
    let _program_path = args.next();
    for arg in args {
        eprintln!("unknown argument: {}", arg);
        std::process::exit(1);
    }
    loop {
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line)? == 0 {
            return Ok(());
        }
        eprintln!("line: {}", line);
        handle::<ast::Ast>(&line)?;
        handle::<ecs::Ecs>(&line)?;
    }
}
