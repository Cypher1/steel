mod primitives;
mod parser;
mod error;
mod ast;

#[cfg(test)]
mod assertions;

use error::SteelErr;
use parser::{hex_color, symbol};

fn main() -> Result<(), SteelErr<'static>> {
    println!("Hello, world!");

    dbg!(hex_color("#2F14DF")?);
    dbg!(symbol("hello   ")?);

    Ok(())
}
