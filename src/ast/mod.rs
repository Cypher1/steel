use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use crate::parser::{ParserContext, ParserStorage};
use std::convert::Infallible;

mod node;
use node::*;

pub struct Ast<'source> {
    members: Arena<Node<'source>>,
    root: Option<Ref<'source>>,
}

impl<'source> Ast<'source> {
    pub fn new() -> Self {
        Self {
            members: Arena::new(),
            root: None,
        }
    }
}

impl<'source> ParserContext<'source> for Ast<'source> {
    type ID = Ref<'source>;
    type E = Infallible;
}

impl<'source> ParserStorage<Ref<'source>, Node<'source>, Infallible> for Ast<'source> {
    fn add(&mut self, value: Node<'source>) -> Ref<'source> {
        let id = self.members.add(value);
        self.members
            .get_mut(id)
            .expect("Getting the 'just' added member, should always be safe")
    }
    fn get(&self, id: Ref<'source>) -> Result<&Node<'source>, Infallible> {
        // This is safe unless a node is deleted... (and we don't expose .remove)
        Ok(unsafe { &*id })
    }
    fn get_mut(&mut self, id: Ref<'source>) -> Result<&mut Node<'source>, Infallible> {
        // This is safe unless a node is deleted... (and we don't expose .remove)
        Ok(unsafe { &mut *id })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_construct_node() {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello").into());

        assert_eq!(
            format!("{:?}", ctx.get(hello)),
            "Ok(Symbol(Symbol { name: \"hello\" }))"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello").into());
        let world = ctx.add(Symbol::new("world").into());

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
        let mut ctx: Ast<'static> = Ast::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]).into());

        assert_eq!(
            format!("{:?}", ctx.get(reference)),
            format!("Ok(Call(Call {{ callee: {:?}, args: [] }}))", reference)
        );
    }
    */

    #[test]
    fn can_construct_nodes_with_cross_reference() -> Result<(), Infallible> {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello").into());
        let world = ctx.add(Symbol::new("world").into());
        let hello: Ref<'static> = ctx.get_mut(hello)?;
        let world: Ref<'static> = ctx.get_mut(world)?;
        let reference = ctx.add(Call::new(hello, vec![world]).into());

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
    fn can_construct_values() -> Result<(), Infallible> {
        let mut ctx: Ast<'static> = Ast::new();

        let plus = ctx.add(Symbol::new("plus").into());
        let a = ctx.add(32i64.into());
        let b = ctx.add(12i64.into());
        let plus: Ref<'static> = ctx.get_mut(plus)?;
        let a: Ref<'static> = ctx.get_mut(a)?;
        let b: Ref<'static> = ctx.get_mut(b)?;
        let reference = ctx.add(Call::new(plus, vec![a, b]).into());

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
