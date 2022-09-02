use nom::error::ParseError;

#[must_use]
#[derive(Debug)]
pub enum SteelErr {
    ParseErrorParseInt(String, std::num::ParseIntError),
    AstError(crate::ast::AstError),
    EcsError(crate::ecs::EcsError),
    PrecedenceError {
        precendence: i32,
    },
    Error {
        input: String,
        code: nom::error::ErrorKind,
    }, // Parse
    Multi(Box<SteelErr>, Box<SteelErr>),
}

impl std::fmt::Display for SteelErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PrecedenceError { precendence } => write!(
                f,
                "Unexpected operator due to max precendence setting ({})",
                precendence
            ),
            ParseErrorParseInt(input, error) => {
                write!(f, "Failed to parse int due to {:?} in {}", error, input)
            }
            Error { input, code } => write!(f, "Failed in {:?} while parsing {}", code, input),
            AstError(e) => write!(f, "{:?}", e),
            EcsError(e) => write!(f, "{:?}", e),
            Multi(a, b) => write!(f, "{}\nand {}", a, b),
        }
    }
}

use SteelErr::*;

impl From<std::convert::Infallible> for SteelErr {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}

impl From<SteelErr> for nom::Err<SteelErr> {
    fn from(err: SteelErr) -> Self {
        nom::Err::Error(err)
    }
}

impl From<nom::Err<SteelErr>> for SteelErr {
    fn from(err: nom::Err<SteelErr>) -> Self {
        match err {
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
            nom::Err::Incomplete(_needed) => todo!("Handle incomplete input"),
        }
    }
}

impl nom::error::FromExternalError<&str, std::num::ParseIntError> for SteelErr {
    fn from_external_error(
        input: &str,
        _kind: nom::error::ErrorKind,
        e: std::num::ParseIntError,
    ) -> Self {
        ParseErrorParseInt(input.to_string(), e)
    }
}

impl ParseError<&str> for SteelErr {
    fn from_error_kind(input: &str, kind: nom::error::ErrorKind) -> Self {
        Error {
            input: input.into(),
            code: kind,
        }
    }
    fn append(input: &str, kind: nom::error::ErrorKind, other: Self) -> Self {
        // TODO: !?
        match other {
            Error { input, code } => Error { input, code },
            _ => Multi(
                Box::new(Self::from_error_kind(input, kind)),
                Box::new(other),
            ),
        }
    }
}

impl From<crate::ast::AstError> for SteelErr {
    fn from(err: crate::ast::AstError) -> Self {
        AstError(err)
    }
}

impl From<crate::ecs::EcsError> for SteelErr {
    fn from(err: crate::ecs::EcsError) -> Self {
        EcsError(err)
    }
}
