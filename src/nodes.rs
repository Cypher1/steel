#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Call<P> {
    pub callee: P,
    pub args: Vec<(String, P)>,
    pub left: Option<P>,
    pub right: Option<P>,
}

impl<P: Clone> Call<P> {
    pub fn new(callee: P, args: Vec<(String, P)>) -> Self {
        let mut left = None;
        let mut right = None;
        for (name, id) in &args {
            if name == "arg_0" {
                left = Some(id).cloned();
            }
            if name == "arg_1" {
                right = Some(id).cloned();
            }
        }
        Self { callee, args, left, right }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
}
