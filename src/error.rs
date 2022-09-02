#[must_use]
#[derive(Debug)]
pub enum SteelErr<'a> {
    ParseError(nom::Err<nom::error::Error<&'a str>>),
    AstError(crate::ast::AstError<'a>),
    EcsError(crate::ecs::EcsError),
}

impl<'a> std::fmt::Display for SteelErr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError(e) => write!(f, "{}", e),
            AstError(e) => write!(f, "{:?}", e),
            EcsError(e) => write!(f, "{:?}", e),
        }
    }
}

use SteelErr::*;

impl<'a> From<std::convert::Infallible> for SteelErr<'a> {
    fn from(err: std::convert::Infallible) -> Self {
        match err {
        }
    }
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for SteelErr<'a> {
    fn from(err: nom::Err<nom::error::Error<&'a str>>) -> Self {
        ParseError(err)
    }
}

impl<'a> From<crate::ast::AstError<'a>> for SteelErr<'a> {
    fn from(err: crate::ast::AstError<'a>) -> Self {
        AstError(err)
    }
}

impl<'a> From<crate::ecs::EcsError> for SteelErr<'a> {
    fn from(err: crate::ecs::EcsError) -> Self {
        EcsError(err)
    }
}
