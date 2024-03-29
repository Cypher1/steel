use crate::tombstoning_arena::Index;
use crate::nodes::*;

#[derive(Clone, Debug)]
pub enum Node {
    I64(i64),
    Operator(Operator),
    Symbol(Symbol),
    Call(Call<Index>),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tombstoning_arena::{Arena, ArenaError};

    #[test]
    fn can_construct_node() {
        let mut ctx: Arena<Node> = Arena::new();

        let hello = ctx.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", ctx.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\" }))"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Arena<Node> = Arena::new();

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

    /*
    #[test]
    fn can_construct_nodes_with_self_reference() {
        // TODO: Work out how to do self references...
        let mut ctx: Arena<Node> = Arena::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!("Ok(Call(Call {{ callee: {:?}, args: [] }}))", reference)
        );
    }
    */

    #[test]
    fn can_construct_nodes_with_cross_reference() -> Result<(), ArenaError> {
        let mut ctx: Arena<Node> = Arena::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));
        let reference = ctx.add(Call::new(hello, vec![("arg_0".to_string(), world)]));

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!(
                "Ok(Call(Call {{ callee: {:?}, args: [(\"arg_0\", {:?})], left: Some({:?}), right: None }}))",
                hello, world, world
            )
        );
        Ok(())
    }

    #[test]
    fn can_construct_values() -> Result<(), ArenaError> {
        let mut ctx: Arena<Node> = Arena::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
        let reference = ctx.add(Call::new(
            plus,
            vec![("arg_0".to_string(), a), ("arg_1".to_string(), b)],
        ));

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!(
                "Ok(Call(Call {{ callee: {:?}, args: [(\"arg_0\", {:?}), (\"arg_1\", {:?})], left: Some({:?}), right: Some({:?}) }}))",
                plus, a, b, a, b
            )
        );
        Ok(())
    }
}
