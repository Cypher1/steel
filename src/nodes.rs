#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol {
    // TODO: Intern strings
    // TODO: Locations
    pub name: String,
    pub is_operator: bool,
}

impl Symbol {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_operator: false,
        }
    }
    pub fn operator(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_operator: true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Call<P> {
    pub callee: P,
    pub args: Vec<(String, P)>,
}

impl<P> Call<P> {
    pub fn new(callee: P, args: Vec<(String, P)>) -> Self {
        Self { callee, args }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
}
