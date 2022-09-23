use crate::arena::{Arena, ID};
use crate::compiler_context::{CompilerContext, NodeStore};
use crate::nodes::*;
use std::marker::PhantomData;

mod component;
use component::*;
mod providers;
use providers::*;

pub use component::EcsError;

// In future there may be other kinds of Providers.
#[macro_use]
mod arena_providers;
use arena_providers::*;

#[derive(Debug, Default)]
pub struct Ecs {
    entities: Arena<Entity>,
    symbols: Arena<Symbol<ID>>,
    calls: Arena<Call<ID>>,
    int64_values: Arena<i64>,
}

make_arena_provider!(Ecs, Symbol<ID>, Symbol, symbols);
make_arena_provider!(Ecs, Call<ID>, Call, calls);
make_arena_provider!(Ecs, i64, I64, int64_values);

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
            + self.int64_values.active_mem_usage()
    }

    fn mem_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entities.mem_usage()
            + self.symbols.mem_usage()
            + self.calls.mem_usage()
            + self.int64_values.mem_usage()
    }
}

impl<T> NodeStore<ID, T, EcsError> for Ecs
where
    Self: Provider<T>,
{
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

        let sym: &Symbol<ID> = ctx.get(hello)?;
        assert_eq!(
            format!("{:?}", sym),
            "Symbol { name: \"hello\", is_operator: false, bound_to: None }"
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
            format!("{:?}", ctx.get::<Symbol<ID>>(hello)),
            "Ok(Symbol { name: \"hello\", is_operator: false, bound_to: None })"
        );
        assert_eq!(
            format!("{:?}", ctx.get::<Symbol<ID>>(world)),
            "Ok(Symbol { name: \"world\", is_operator: false, bound_to: None })"
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
        let reference = ctx.add(Call::new(hello, vec![world]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [{:?}] }})", hello, world)
        );
    }

    #[test]
    fn can_construct_values() {
        let mut ctx: Ecs = Ecs::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
        let reference = ctx.add(Call::new(plus, vec![a, b]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [{:?}, {:?}] }})",
                plus, a, b
            )
        );
    }
}
