mod primitives;
mod parser;
mod error;

use error::SteelErr;
use parser::hex_color;

fn main() -> Result<(), SteelErr<'static>> {
    println!("Hello, world!");

    dbg!(hex_color("#2F14DF")?);

    Ok(())
}
