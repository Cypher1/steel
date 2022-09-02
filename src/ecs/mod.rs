use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use crate::parser::{ParserContext, ParserStorage};
use std::marker::PhantomData;

mod component;
use component::*;
mod providers;
use providers::*;

// In future there may be other kinds of Providers.
#[macro_use]
mod arena_providers;
use arena_providers::*;

#[derive(Debug, Default)]
pub struct ECS<'source> {
    entities: Arena<Entity>,
    symbols: Arena<Symbol<'source>>,
    calls: Arena<Call<ID>>,
    int64_values: Arena<i64>,
}

make_arena_provider!(ECS<'a>, Symbol<'a>, Symbol, symbols);
make_arena_provider!(ECS<'a>, Call<ID>, Call, calls);
make_arena_provider!(ECS<'a>, i64, I64, int64_values);

impl<'source> ParserContext<'source> for ECS<'source> {
    type ID = ID;
    type E = ECSError;
}

impl<'source, T: 'source> ParserStorage<ID, T, ECSError> for ECS<'source>
where
    Self: Provider<'source, T>,
{
    fn add(&mut self, value: T) -> ID {
        self.add_component(value)
    }

    fn get(&self, id: ID) -> Result<&T, ECSError> {
        <ECS<'source> as Provider<'source, T>>::get_component_for_entity(self, id)
    }
    #[allow(unused)]
    fn get_mut(&mut self, id: ID) -> Result<&mut T, ECSError>
    where
        Self: Provider<'source, T>,
    {
        <ECS<'source> as Provider<'source, T>>::get_component_for_entity_mut(self, id)
    }
}

impl<'source> ECS<'source> {
    pub fn new() -> Self {
        Default::default()
    }

    fn add<T>(&mut self, value: T) -> ID
    where
        Self: ParserStorage<ID, T, ECSError>,
    {
        <Self as ParserStorage<ID, T, ECSError>>::add(self, value)
    }

    fn get<T>(&self, id: ID) -> Result<&T, ECSError>
    where
        Self: ParserStorage<ID, T, ECSError>,
    {
        <Self as ParserStorage<ID, T, ECSError>>::get(self, id)
    }

    fn get_mut<T>(&mut self, id: ID) -> Result<&mut T, ECSError>
    where
        Self: ParserStorage<ID, T, ECSError>,
    {
        <Self as ParserStorage<ID, T, ECSError>>::get_mut(self, id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    type Call = super::Call<ID>;

    #[test]
    fn can_construct_node() -> Result<(), ECSError> {
        let mut ctx: ECS<'static> = ECS::new();

        let hello = ctx.add(Symbol::new("hello"));

        let sym: &Symbol<'static> = ctx.get(hello)?;
        assert_eq!(format!("{:?}", sym), "Symbol { name: \"hello\" }");
        Ok(())
    }

    #[test]
    fn cannot_access_incorrect_node() {
        let mut ctx: ECS<'static> = ECS::new();
        let hello = ctx.add(Symbol::new("hello"));
        let call: Result<&Call, ECSError> = ctx.get(hello);
        assert_eq!(
            format!("{:?}", call),
            format!("Err(ComponentNotFound({:?}))", hello)
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: ECS<'static> = ECS::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get::<Symbol>(hello)),
            "Ok(Symbol { name: \"hello\" })"
        );
        assert_eq!(
            format!("{:?}", ctx.get::<Symbol>(world)),
            "Ok(Symbol { name: \"world\" })"
        );
    }

    #[test]
    fn can_construct_nodes_with_self_reference() {
        let mut ctx: ECS<'static> = ECS::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [] }})", reference)
        );
    }

    #[test]
    fn can_construct_nodes_with_cross_reference() {
        let mut ctx: ECS<'static> = ECS::new();

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
        let mut ctx: ECS<'static> = ECS::new();

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
