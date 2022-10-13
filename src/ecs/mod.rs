use crate::arena::{Arena, ID};
use crate::compiler_context::{CompilerContext, ForEachNode, NodeStore};
use crate::nodes::*;
use std::marker::PhantomData;

mod component;
use component::*;
mod providers;
use providers::*;

pub use providers::EcsError;

// In future there may be other kinds of Providers.
#[macro_use]
mod arena_providers;
use arena_providers::*;

#[derive(Debug, Default)]
pub struct Ecs {
    entities: Arena<Entity>,
    symbols: Arena<(ID, Symbol)>,
    calls: Arena<(ID, Call<ID>)>,
    i64_values: Arena<(ID, i64)>,
}

make_arena_provider!(Ecs, Symbol, symbol, symbols);
make_arena_provider!(Ecs, Call<ID>, call, calls);
make_arena_provider!(Ecs, i64, i_64, i64_values);

impl CompilerContext for Ecs {
    type ID = ID;
    type E = EcsError;
    fn new() -> Self {
        Self::new()
    }

    fn active_mem_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entities.active_mem_usage()
            + self.symbols.active_mem_usage()
            + self.calls.active_mem_usage()
            + self.i64_values.active_mem_usage()
    }

    fn mem_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entities.mem_usage()
            + self.symbols.mem_usage()
            + self.calls.mem_usage()
            + self.i64_values.mem_usage()
    }

    fn for_each(
        &mut self,
        symbol_fn: ForEachNode<Self, Symbol>,
        call_fn: ForEachNode<Self, Call<Self::ID>>,
        i64_fn: ForEachNode<Self, i64>,
    ) -> Result<(), Self::E> {
        // TODO: Parallel?
        for (id, i64_value) in &mut self.i64_values {
            i64_fn(*id, i64_value, &mut self.entities.get_mut(*id)?.shared);
        }
        for (id, symbol) in &mut self.symbols {
            symbol_fn(*id, symbol, &mut self.entities.get_mut(*id)?.shared);
        }
        for (id, call) in &mut self.calls {
            call_fn(*id, call, &mut self.entities.get_mut(*id)?.shared);
        }
        Ok(())
    }
}

impl NodeStore<ID, Shared<ID>, EcsError> for Ecs {
    fn replace(&mut self, id: ID, _value: Shared<ID>) -> Result<(), EcsError> {
        panic!("Don't replace shared data on it's own")
    }
    fn add(&mut self, _value: Shared<ID>) -> ID {
        panic!("Don't add shared data on it's own")
    }

    fn get(&self, id: ID) -> Result<&Shared<ID>, EcsError> {
        Ok(&self.entities.get(id)?.shared)
    }

    fn get_mut(&mut self, id: ID) -> Result<&mut Shared<ID>, EcsError> {
        Ok(&mut self.entities.get_mut(id)?.shared)
    }
}
impl<T> NodeStore<ID, T, EcsError> for Ecs
where
    Self: Provider<T>,
{
    fn replace(&mut self, id: ID, value: T) -> Result<(), EcsError> {
        self.replace_component(id, value)?;
        Ok(())
    }

    fn add(&mut self, value: T) -> ID {
        self.add_component(value)
    }

    fn get(&self, id: ID) -> Result<&T, EcsError> {
        <Ecs as Provider<T>>::get_component_for_entity(self, id)
    }
    #[allow(unused)]
    fn get_mut(&mut self, id: ID) -> Result<&mut T, EcsError>
    where
        Self: Provider<T>,
    {
        <Ecs as Provider<T>>::get_component_for_entity_mut(self, id)
    }
}

impl Ecs {
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(test)]
    fn add<T>(&mut self, value: T) -> ID
    where
        Self: NodeStore<ID, T, EcsError>,
    {
        <Self as NodeStore<ID, T, EcsError>>::add(self, value)
    }

    #[cfg(test)]
    fn get<T>(&self, id: ID) -> Result<&T, EcsError>
    where
        Self: NodeStore<ID, T, EcsError>,
    {
        <Self as NodeStore<ID, T, EcsError>>::get(self, id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    type Call = super::Call<ID>;

    #[test]
    fn can_construct_node() -> Result<(), EcsError> {
        let mut ctx: Ecs = Ecs::new();

        let hello = ctx.add(Symbol::new("hello"));

        let sym: &Symbol = ctx.get(hello)?;
        assert_eq!(
            format!("{:?}", sym),
            "Symbol { name: \"hello\", is_operator: false }"
        );
        Ok(())
    }

    #[test]
    fn cannot_access_incorrect_node() {
        let mut ctx: Ecs = Ecs::new();
        let hello = ctx.add(Symbol::new("hello"));
        let call: Result<&Call, EcsError> = ctx.get(hello);
        assert_eq!(
            format!("{:?}", call),
            format!("Err(ComponentNotFound({:?}))", hello)
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Ecs = Ecs::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get::<Symbol>(hello)),
            "Ok(Symbol { name: \"hello\", is_operator: false })"
        );
        assert_eq!(
            format!("{:?}", ctx.get::<Symbol>(world)),
            "Ok(Symbol { name: \"world\", is_operator: false })"
        );
    }

    #[test]
    fn can_construct_nodes_with_self_reference() {
        let mut ctx: Ecs = Ecs::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [] }})", reference)
        );
    }

    #[test]
    fn can_construct_nodes_with_cross_reference() {
        let mut ctx: Ecs = Ecs::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));
        let reference = ctx.add(Call::new(hello, vec![("arg_0".to_string(), world)]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [(\"arg_0\", {:?})] }})",
                hello, world
            )
        );
    }

    #[test]
    fn can_construct_values() {
        let mut ctx: Ecs = Ecs::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
        let reference = ctx.add(Call::new(
            plus,
            vec![("arg_0".to_string(), a), ("arg_1".to_string(), b)],
        ));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [(\"arg_0\", {:?}), (\"arg_1\", {:?})] }})",
                plus, a, b
            )
        );
    }
}
