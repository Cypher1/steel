use crate::arena::{Arena, ArenaError, Index};
use crate::compiler_context::{CompilerContext, ForEachNode, NodeStore};
use crate::nodes::*;

mod node;
use node::*;

#[derive(Debug)]
pub enum AstError {
    NodeOfWrongKindError(Index, &'static str),
    InternalError(ArenaError),
}

impl From<ArenaError> for AstError {
    fn from(it: ArenaError) -> Self {
        InternalError(it)
    }
}

use AstError::*;

#[derive(Debug, Default)]
pub struct Ast {
    members: Arena<(Node, Shared<Index>)>,
}

impl Ast {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CompilerContext for Ast
where
    Self: NodeStore<Index, i64, AstError>,
    Self: NodeStore<Index, Symbol, AstError>,
    Self: NodeStore<Index, Call<Index>, AstError>,
    Self: NodeStore<Index, Shared<Index>, AstError>,
{
    type ID = Index;
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

    fn for_each(
        &mut self,
        symbol_fn: ForEachNode<Self, Symbol>,
        call_fn: ForEachNode<Self, Call<Self::ID>>,
        i64_fn: ForEachNode<Self, i64>,
    ) -> Result<(), Self::E> {
        for (id, node) in (&mut self.members).into_iter().enumerate() {
            match &mut node.0 {
                Node::Symbol(symbol) => symbol_fn(id, symbol, &mut node.1),
                Node::Call(call) => call_fn(id, call, &mut node.1),
                Node::I64(value) => i64_fn(id, value, &mut node.1),
            }
        }
        Ok(())
    }
}

impl NodeStore<Index, Node, ArenaError> for Ast {
    fn overwrite(&mut self, id: Index, value: Node) -> Result<Option<Node>, ArenaError> {
        self.members.set(id, (value, Shared::default()))?;
        Ok(None)
    }

    fn remove(&mut self, id: Index) -> Result<Option<Node>, ArenaError> {
        Ok(self.members.remove(id)?.map(|op| op.0))
    }
    fn add(&mut self, value: Node) -> Index {
        self.members.add((value, Shared::default()))
    }
    fn get(&self, id: Index) -> Result<&Node, ArenaError> {
        Ok(&self.members.get(id)?.0)
    }
    fn get_mut(&mut self, id: Index) -> Result<&mut Node, ArenaError> {
        Ok(&mut self.members.get_mut(id)?.0)
    }
}

impl NodeStore<Index, Shared<Index>, AstError> for Ast {
    fn remove(&mut self, id: Index) -> Result<Option<Shared<Index>>, AstError> {
        Ok(self.members.remove(id)?.map(|op| op.1))
    }
    fn add(&mut self, _value: Shared<Index>) -> Index {
        panic!("Don't add shared data on it's own")
    }
    fn get(&self, id: Index) -> Result<&Shared<Index>, AstError> {
        Ok(&self.members.get(id)?.1)
    }
    fn get_mut(&mut self, id: Index) -> Result<&mut Shared<Index>, AstError> {
        Ok(&mut self.members.get_mut(id)?.1)
    }
}

macro_rules! wrap_node {
    ($ty: ty, $variant: tt) => {
        impl From<$ty> for Node {
            fn from(it: $ty) -> Self {
                Node::$variant(it)
            }
        }
        impl NodeStore<Index, $ty, AstError> for Ast {
            fn overwrite(&mut self, id: Index, value: $ty) -> Result<Option<$ty>, AstError> {
                let value = std::convert::Into::<Node>::into(value);
                self.overwrite(id, value)?;
                Ok(None)
            }

            fn remove(&mut self, id: Index) -> Result<Option<$ty>, AstError> {
                let result = <Self as NodeStore<Index, Node, ArenaError>>::remove(self, id)?;
                if let Some(Node::$variant(value)) = result {
                    Ok(Some(value))
                } else if result.is_none() {
                    Ok(None)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
            fn add(&mut self, value: $ty) -> Index {
                self.add(std::convert::Into::<Node>::into(value))
            }
            fn get(&self, id: Index) -> Result<&$ty, AstError> {
                if let Node::$variant(ref value) =
                    <Self as NodeStore<Index, Node, ArenaError>>::get(self, id)?
                {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
            fn get_mut(&mut self, id: Index) -> Result<&mut $ty, AstError> {
                if let Node::$variant(ref mut value) =
                    <Self as NodeStore<Index, Node, ArenaError>>::get_mut(self, id)?
                {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
        }
    };
}

wrap_node!(Symbol, Symbol);
wrap_node!(Call<Index>, Call);
wrap_node!(i64, I64);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_construct_node() {
        let mut ctx: Ast = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", ctx.get_symbol(hello)),
            "Ok(Symbol { name: \"hello\", is_operator: false })"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Ast = Ast::new();

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
        let mut ctx: Ast = Ast::new();

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
        let mut ctx: Ast = Ast::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [] }})", reference)
        );
    }
    */

    #[test]
    fn can_construct_nodes_with_cross_reference() -> Result<(), ArenaError> {
        let mut ctx: Ast = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));
        let reference = ctx.add(Call::new(hello, vec![("arg_0".to_string(), world)]));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [(\"arg_0\", {:?})] }})",
                hello, world
            )
        );
        Ok(())
    }

    #[test]
    fn can_construct_values() -> Result<(), ArenaError> {
        let mut ctx: Ast = Ast::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
        let reference = ctx.add(Call::new(
            plus,
            vec![("arg_0".to_string(), a), ("arg_1".to_string(), b)],
        ));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [(\"arg_0\", {:?}), (\"arg_1\", {:?})] }})",
                plus, a, b
            )
        );
        Ok(())
    }
}
