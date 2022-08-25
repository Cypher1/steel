use crate::arena::{/*Arena,*/ ID};

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol<'a> {
    // TODO: Intern strings
    // TODO: Locations
    pub name: &'a str,
}

impl<'a> Symbol<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Call {
    pub callee: ID,
    pub args: Vec<ID>,
}

impl Call {
    pub fn new(callee: ID, args: Vec<ID>) -> Self {
        Self { callee, args }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
}
