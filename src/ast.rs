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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Node<'a> {
    Symbol(Symbol<'a>),
    Call(Call),
}

pub fn symbol<'a>(name: &'a str) -> Node<'a> {
    Node::Symbol(Symbol::new(name))
}

pub fn call<'a>(id: ID, args: Vec<ID>) -> Node<'a> {
    Node::Call(Call::new(id, args))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_construct_node() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let hello = a.add(symbol("hello"));

        assert_eq!(format!("{:?}", a.get(hello)), "Ok(Symbol(Symbol { name: \"hello\" }))");
    }

    #[test]
    fn can_construct_nodes() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let hello = a.add(symbol("hello"));
        let world = a.add(symbol("world"));

        assert_eq!(format!("{:?}", a.get(hello)), "Ok(Symbol(Symbol { name: \"hello\" }))");
        assert_eq!(format!("{:?}", a.get(world)), "Ok(Symbol(Symbol { name: \"world\" }))");
    }

    #[test]
    fn can_construct_nodes_with_self_reference() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let reference = a.add_with_id(|id|call(id, vec![]));

        assert_eq!(format!("{:?}", a.get(reference)), format!("Ok(Call(Call {{ callee: {:?}, args: [] }}))", reference));
    }

    #[test]
    fn can_construct_nodes_with_cross_reference() {
        let mut a: Arena<Node<'static>> = Arena::new();

        let hello = a.add(symbol("hello"));
        let world = a.add(symbol("world"));
        let reference = a.add(call(hello, vec![world]));

        assert_eq!(format!("{:?}", a.get(reference)), format!("Ok(Call(Call {{ callee: {:?}, args: [{:?}] }}))", hello, world));
    }
}
