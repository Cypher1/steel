#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol<'a, P> {
    // TODO: Intern strings
    // TODO: Locations
    pub name: &'a str,
    pub is_operator: bool,
    pub bound_to: Option<P>,
}

impl<'a, P> Symbol<'a, P> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            is_operator: false,
            bound_to: None,
        }
    }
    pub fn operator(name: &'a str) -> Self {
        Self {
            name,
            is_operator: true,
            bound_to: None,
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
