use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use crate::compiler_context::{CompilerContext, NodeStore};

mod node;
use node::*;

#[derive(Debug)]
pub enum AstError {
    NodeOfWrongKindError(ID, &'static str),
    InternalError(ArenaError),
}

impl From<ArenaError> for AstError {
    fn from(it: ArenaError) -> Self {
        InternalError(it)
    }
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

impl<'source> CompilerContext<'source> for Ast<'source>
where
    Self: NodeStore<'source, ID, i64, AstError>,
    Self: NodeStore<'source, ID, Symbol<'source>, AstError>,
    Self: NodeStore<'source, ID, Call<ID>, AstError>,
{
    type ID = ID;
    type E = AstError;

    fn new() -> Self {
        Self::new()
    }

    fn active_mem_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.members.active_mem_usage()
    }

    fn mem_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.members.mem_usage()
    }
}

impl<'source> NodeStore<'source, ID, Node<'source>, ArenaError> for Ast<'source> {
    fn add(&mut self, value: Node<'source>) -> ID {
        self.members.add(value)
    }
    fn get(&self, id: ID) -> Result<&Node<'source>, ArenaError> {
        self.members.get(id)
    }
    fn get_mut(&mut self, id: ID) -> Result<&mut Node<'source>, ArenaError> {
        self.members.get_mut(id)
    }
}

macro_rules! wrap_node {
    ($ty: ty, $variant: tt) => {
        impl<'source> From<$ty> for Node<'source> {
            fn from(it: $ty) -> Self {
                Node::$variant(it)
            }
        }
        impl<'source> NodeStore<'source, ID, $ty, AstError> for Ast<'source> {
            fn add(&mut self, value: $ty) -> ID {
                self.add(std::convert::Into::<Node<'source>>::into(value))
            }
            fn get(&self, id: ID) -> Result<&$ty, AstError> {
                if let Node::$variant(ref value) =
                    <Self as NodeStore<'source, ID, Node<'source>, ArenaError>>::get(self, id)?
                {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
            fn get_mut(&mut self, id: ID) -> Result<&mut $ty, AstError> {
                if let Node::$variant(ref mut value) =
                    <Self as NodeStore<'source, ID, Node<'source>, ArenaError>>::get_mut(
                        self, id,
                    )?
                {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
        }
    };
}

wrap_node!(Symbol<'source>, Symbol);
wrap_node!(Call<ID>, Call);
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
    fn can_construct_nodes_with_cross_reference() -> Result<(), ArenaError> {
        let mut ctx: Ast<'static> = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));
        let reference = ctx.add(Call::new(hello, vec![world]));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [{:?}] }})", hello, world)
        );
        Ok(())
    }

    #[test]
    fn can_construct_values() -> Result<(), ArenaError> {
        let mut ctx: Ast<'static> = Ast::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
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
