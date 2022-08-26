use crate::arena::ID;

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
pub struct Call<P> {
    pub callee: P,
    pub args: Vec<P>,
}

impl<P> Call<P> {
    pub fn new(callee: P, args: Vec<P>) -> Self {
        Self { callee, args }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
}
