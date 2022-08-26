use crate::nodes::*;

impl<'a> From<Symbol<'a>> for Node<'a> {
    fn from(it: Symbol<'a>) -> Self {
        Node::Symbol(it)
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
    use crate::arena::Arena;

    #[test]
    fn can_construct_node() {
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let hello = ctx.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", ctx.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\" }))"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\" }))"
        );
        assert_eq!(
            format!("{:?}", ctx.get(world)),
            "Ok(Symbol(Symbol { name: \"world\" }))"
        );
    }

    #[test]
    fn can_construct_nodes_with_self_reference() {
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!("Ok(Call(Call {{ callee: {:?}, args: [] }}))", reference)
        );
    }

    #[test]
    fn can_construct_nodes_with_cross_reference() {
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));
        let reference = ctx.add(Call::new(hello, vec![world]));

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!(
                "Ok(Call(Call {{ callee: {:?}, args: [{:?}] }}))",
                hello, world
            )
        );
    }

    #[test]
    fn can_construct_values() {
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
        let reference = ctx.add(Call::new(plus, vec![a, b]));

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!(
                "Ok(Call(Call {{ callee: {:?}, args: [{:?}, {:?}] }}))",
                plus, a, b
            )
        );
    }
}
