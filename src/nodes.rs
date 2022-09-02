#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol<'a> {
    // TODO: Intern strings
    // TODO: Locations
    pub name: &'a str,
    pub is_operator: bool,
}

impl<'a> Symbol<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            is_operator: false,
        }
    }
    pub fn operator(name: &'a str) -> Self {
        Self {
            name,
            is_operator: true,
        }
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
