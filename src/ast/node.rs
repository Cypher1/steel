use crate::arena::ID;
use crate::nodes::*;

#[derive(Debug)]
pub enum Node<'source> {
    Symbol(Symbol<'source>),
    Call(Call<ID>),
    I64(i64),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arena::{Arena, ArenaError};

    #[test]
    fn can_construct_node() {
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let hello = ctx.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", ctx.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\", is_operator: false }))"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\", is_operator: false }))"
        );
        assert_eq!(
            format!("{:?}", ctx.get(world)),
            "Ok(Symbol(Symbol { name: \"world\", is_operator: false }))"
        );
    }

    /*
    #[test]
    fn can_construct_nodes_with_self_reference() {
        // TODO: Work out how to do self references...
        let mut ctx: Arena<Node<'static>> = Arena::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!("Ok(Call(Call {{ callee: {:?}, args: [] }}))", reference)
        );
    }
    */

    #[test]
    fn can_construct_nodes_with_cross_reference() -> Result<(), ArenaError> {
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
        Ok(())
    }

    #[test]
    fn can_construct_values() -> Result<(), ArenaError> {
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
        Ok(())
    }
}
