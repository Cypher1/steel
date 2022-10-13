use crate::arena::{Arena, ArenaError, ID};
use crate::compiler_context::{CompilerContext, ForEachNode, NodeStore};
use crate::nodes::*;

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

#[derive(Debug, Default)]
pub struct Ast {
    members: Arena<Node>,
}

impl Ast {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CompilerContext for Ast
where
    Self: NodeStore<ID, I64Value<ID>, AstError>,
    Self: NodeStore<ID, Symbol<ID>, AstError>,
    Self: NodeStore<ID, Call<ID>, AstError>,
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

    fn for_each(
        &mut self,
        symbol_fn: ForEachNode<Self, Symbol<Self::ID>>,
        call_fn: ForEachNode<Self, Call<Self::ID>>,
        i64_fn: ForEachNode<Self, I64Value<Self::ID>>,
    ) {
        for (id, node) in (&mut self.members).into_iter().enumerate() {
            match node {
                Node::Symbol(node) => symbol_fn(id, node),
                Node::Call(node) => call_fn(id, node),
                Node::I64(node) => i64_fn(id, node),
            }
        }
    }
}

impl NodeStore<ID, Node, ArenaError> for Ast {
    fn add(&mut self, value: Node) -> ID {
        self.members.add(value)
    }
    fn get(&self, id: ID) -> Result<&Node, ArenaError> {
        Ok(self.members.get(id)?)
    }
    fn get_mut(&mut self, id: ID) -> Result<&mut Node, ArenaError> {
        Ok(self.members.get_mut(id)?)
    }
}

macro_rules! wrap_node {
    ($ty: ty, $variant: tt) => {
        impl From<$ty> for Node {
            fn from(it: $ty) -> Self {
                Node::$variant(it)
            }
        }
        impl NodeStore<ID, $ty, AstError> for Ast {
            fn add(&mut self, value: $ty) -> ID {
                self.add(std::convert::Into::<Node>::into(value))
            }
            fn get(&self, id: ID) -> Result<&$ty, AstError> {
                if let Node::$variant(ref value) =
                    <Self as NodeStore<ID, Node, ArenaError>>::get(self, id)?
                {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
            fn get_mut(&mut self, id: ID) -> Result<&mut $ty, AstError> {
                if let Node::$variant(ref mut value) =
                    <Self as NodeStore<ID, Node, ArenaError>>::get_mut(self, id)?
                {
                    Ok(value)
                } else {
                    Err(NodeOfWrongKindError(id, stringify!($variant)))
                }
            }
        }
    };
}

wrap_node!(Symbol<ID>, Symbol);
wrap_node!(Call<ID>, Call);
wrap_node!(I64Value<ID>, I64);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_construct_node() {
        let mut ctx: Ast = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));

        assert_eq!(
            format!("{:?}", ctx.get_symbol(hello)),
            "Ok(Symbol { name: \"hello\", is_operator: false, shared: {} })"
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Ast = Ast::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get_symbol(hello)),
            "Ok(Symbol { name: \"hello\", is_operator: false, shared: {} })"
        );
        assert_eq!(
            format!("{:?}", ctx.get_symbol(world)),
            "Ok(Symbol { name: \"world\", is_operator: false, shared: {} })"
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
            format!("Ok(Call {{ callee: {:?}, args: [], shared: {{}} }})", reference)
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
                "Ok(Call {{ callee: {:?}, args: [(\"arg_0\", {:?})], shared: {{}} }})",
                hello, world
            )
        );
        Ok(())
    }

    #[test]
    fn can_construct_values() -> Result<(), ArenaError> {
        let mut ctx: Ast = Ast::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(I64Value::from(32i64));
        let b = ctx.add(I64Value::from(12i64));
        let reference = ctx.add(Call::new(
            plus,
            vec![("arg_0".to_string(), a), ("arg_1".to_string(), b)],
        ));

        assert_eq!(
            format!("{:?}", ctx.get_call(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [(\"arg_0\", {:?}), (\"arg_1\", {:?})], shared: {{}} }})",
                plus, a, b
            )
        );
        Ok(())
    }
}
