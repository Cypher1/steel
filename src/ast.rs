use crate::arena::{Arena, ID};

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol<'a> {
    // TODO: Intern strings
    // TODO: Locations
    pub name: &'a str,
}

impl<'a> Symbol<'a> {
    fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl<'a> From<Symbol<'a>> for Node<'a> {
    fn from(it: Symbol<'a>) -> Self {
        Node::Symbol(it)
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Call {
    pub callee: ID,
    pub args: Vec<ID>,
}

impl Call {
    fn new(callee: ID, args: Vec<ID>) -> Self {
        Self { callee, args }
    }
}

impl<'a> From<Call> for Node<'a> {
    fn from(it: Call) -> Self {
        Node::Call(it)
    }
}

impl<'a> From<i64> for Node<'a> {
    fn from(it: i64) -> Self {
        Node::I64(it)
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Node<'a> {
    Symbol(Symbol<'a>),
    Call(Call),
    I64(i64),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_construct_node() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let hello = a.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", a.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\" }))"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let hello = a.add(Symbol::new("hello"));
        let world = a.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", a.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\" }))"
        );
        assert_eq!(
            format!("{:?}", a.get(world)),
            "Ok(Symbol(Symbol { name: \"world\" }))"
        );
    }

    #[test]
    fn can_construct_nodes_with_self_reference() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let reference = a.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", a.get(reference)),
            format!("Ok(Call(Call {{ callee: {:?}, args: [] }}))", reference)
        );
    }

    #[test]
    fn can_construct_nodes_with_cross_reference() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let hello = a.add(Symbol::new("hello"));
        let world = a.add(Symbol::new("world"));
        let reference = a.add(Call::new(hello, vec![world]));

        assert_eq!(
            format!("{:?}", a.get(reference)),
            format!(
                "Ok(Call(Call {{ callee: {:?}, args: [{:?}] }}))",
                hello, world
            )
        );
    }
}
