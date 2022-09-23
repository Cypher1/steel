#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol<P> {
    // TODO: Intern strings
    // TODO: Locations
    pub name: String,
    pub is_operator: bool,
    pub bound_to: Option<P>,
}

impl<P> Symbol<P> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_operator: false,
            bound_to: None,
        }
    }
    pub fn operator(name: &str) -> Self {
        Self {
            name: name.to_string(),
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
