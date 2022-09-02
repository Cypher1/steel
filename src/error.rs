#[must_use]
#[derive(Debug)]
pub enum SteelErr<'a> {
    ParseError(nom::Err<nom::error::Error<&'a str>>),
}

impl<'a> std::fmt::Display for SteelErr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError(e) => write!(f, "{}", e),
        }
    }
}

use SteelErr::*;

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for SteelErr<'a> {
    fn from(err: nom::Err<nom::error::Error<&'a str>>) -> Self {
        ParseError(err)
    }
}
