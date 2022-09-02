use crate::arena::Arena;
use crate::nodes::*;
use crate::parser::{ParserContext, ParserStorage};
use std::convert::Infallible;

mod node;
use node::*;

#[derive(Debug)]
pub enum AstError<'source> {
    NodeOfWrongKindError(Ref<'source>, &'static str),
}
use AstError::*;

pub struct Ast<'source> {
    members: Arena<Node<'source>>,
}

impl<'source> Ast<'source> {
    pub fn new() -> Self {
        Self {
            members: Arena::new(),
        }
    }
}

impl<'source> ParserContext<'source> for Ast<'source>
where
    Self: ParserStorage<'source, Ref<'source>, i64, AstError<'source>>,
    Self: ParserStorage<'source, Ref<'source>, Symbol<'source>, AstError<'source>>,
    Self: ParserStorage<'source, Ref<'source>, Call<Ref<'source>>, AstError<'source>>,
{
    type ID = Ref<'source>;
    type E = AstError<'source>;

    fn active_mem_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.members.active_mem_usage()
    }

    fn mem_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.members.mem_usage()
    }
}

impl<'source> ParserStorage<'source, Ref<'source>, Node<'source>, Infallible> for Ast<'source> {
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

macro_rules! wrap_node {
    ($ty: ty, $variant: tt) => {
        impl<'source> From<$ty> for Node<'source> {
            fn from(it: $ty) -> Self {
                Node::$variant(it)
            }
        }
        impl<'source> ParserStorage<'source, Ref<'source>, $ty, AstError<'source>> for Ast<'source> {
            fn add(&mut self, value: $ty) -> Ref<'source> {
                self.add(std::convert::Into::<Node<'source>>::into(value))
            }
            fn get(&self, id: Ref<'source>) -> Result<&$ty, AstError<'source>> {
                if let Node::$variant(ref value) = unsafe { &*id } {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
            fn get_mut(&mut self, id: Ref<'source>) -> Result<&mut $ty, AstError<'source>> {
                if let Node::$variant(ref mut value) = unsafe { &mut *id } {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
        }
    };
}

wrap_node!(Symbol<'source>, Symbol);
wrap_node!(Call<Ref<'source>>, Call);
wrap_node!(i64, I64);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_construct_node() {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", ctx.get_symbol(hello)),
            "Ok(Symbol { name: \"hello\", is_operator: false })"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get_symbol(hello)),
            "Ok(Symbol { name: \"hello\", is_operator: false })"
        );
        assert_eq!(
            format!("{:?}", ctx.get_symbol(world)),
            "Ok(Symbol { name: \"world\", is_operator: false })"
        );
    }

    #[test]
    fn canot_accidentally_cast_to_different_node_type() {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", ctx.get_call(hello)),
            format!("Err(NodeOfWrongKindError({:?}, \"Call\"))", hello)
        );
    }

    /*
    #[test]
    fn can_construct_nodes_with_self_reference() {
        // TODO: Work out how to do self references...
        let mut ctx: Ast<'static> = Ast::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [] }})", reference)
        );
    }
    */

    #[test]
    fn can_construct_nodes_with_cross_reference() -> Result<(), Infallible> {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));
        let hello: Ref<'static> = ctx.get_mut(hello)?;
        let world: Ref<'static> = ctx.get_mut(world)?;
        let reference = ctx.add(Call::new(hello, vec![world]));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [{:?}] }})", hello, world)
        );
        Ok(())
    }

    #[test]
    fn can_construct_values() -> Result<(), Infallible> {
        let mut ctx: Ast<'static> = Ast::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
        let plus: Ref<'static> = ctx.get_mut(plus)?;
        let a: Ref<'static> = ctx.get_mut(a)?;
        let b: Ref<'static> = ctx.get_mut(b)?;
        let reference = ctx.add(Call::new(plus, vec![a, b]));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [{:?}, {:?}] }})",
                plus, a, b
            )
        );
        Ok(())
    }
}
