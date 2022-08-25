#[derive(Debug)]
pub enum SteelErr<'a> {
    ParseError(nom::Err<nom::error::Error<&'a str>>),
}

use SteelErr::*;

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for SteelErr<'a> {
    fn from(err: nom::Err<nom::error::Error<&'a str>>) -> Self {
        ParseError(err)
    }
}
