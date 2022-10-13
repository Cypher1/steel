#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct OptimizerData<P> {
    pub equivalent_to: Vec<P>,
}
impl<P> Default for OptimizerData<P> {
    fn default() -> Self {
        Self {
            equivalent_to: Default::default(),
        }
    }
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Shared<P> {
    optimizer_data: OptimizerData<P>,
}

impl<P> std::fmt::Debug for Shared<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{}}")
    }
}

impl<P> Default for Shared<P> {
    fn default() -> Self {
        Self {
            optimizer_data: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol<P> {
    // TODO: Intern strings
    // TODO: Locations
    pub name: String,
    pub is_operator: bool,
    pub shared: Shared<P>,
}

impl<P> Symbol<P> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_operator: false,
            shared: Shared::default(),
        }
    }
    pub fn operator(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_operator: true,
            shared: Shared::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Call<P> {
    pub callee: P,
    pub args: Vec<(String, P)>,
    pub shared: Shared<P>,
}

impl<P> Call<P> {
    pub fn new(callee: P, args: Vec<(String, P)>) -> Self {
        Self {
            callee,
            args,
            shared: Shared::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct I64Value<P> {
    pub value: i64,
    pub shared: Shared<P>,
}

impl<P> I64Value<P> {
    pub fn new(value: i64) -> Self {
        Self {
            value,
            shared: Shared::default(),
        }
    }
}

impl<P> From<i64> for I64Value<P> {
    fn from(value: i64) -> Self {
        Self {
            value,
            shared: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
}
