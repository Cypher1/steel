use crate::arena::Arena;
use crate::compiler_context::{CompilerContext, NodeStore};
use crate::nodes::*;

mod component;
use component::*;
mod providers;
use providers::*;

pub use providers::EcsError;

// In future there may be other kinds of Providers.
#[macro_use]
mod arena_providers;
use arena_providers::*;

#[derive(Clone, Debug, Default)]
pub struct Ecs {
    entities: Arena<Entity>,
    i64_values: Arena<(EntityId, i64)>,
    operators: Arena<(EntityId, Operator)>,
    symbols: Arena<(EntityId, Symbol)>,
    calls: Arena<(EntityId, Call<EntityId>)>,
}

make_arena_provider!(Ecs, i64, i_64, i64_values);
make_arena_provider!(Ecs, Operator, operator, operators);
make_arena_provider!(Ecs, Symbol, symbol, symbols);
make_arena_provider!(Ecs, Call<EntityId>, call, calls);

impl CompilerContext for Ecs {
    type ID = EntityId;
    type E = EcsError;
    fn new() -> Self {
        Self::new()
    }

    fn active_mem_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entities.active_mem_usage()
            + self.i64_values.active_mem_usage()
            + self.operators.active_mem_usage()
            + self.symbols.active_mem_usage()
            + self.calls.active_mem_usage()
    }

    fn mem_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entities.mem_usage()
            + self.i64_values.mem_usage()
            + self.operators.mem_usage()
            + self.symbols.mem_usage()
            + self.calls.mem_usage()
    }

    fn for_each_i64<F: FnMut(Self::ID, &mut i64)>(&mut self, f: &mut F) -> Result<(), Self::E> {
        for (id, i64_value) in &mut self.i64_values {
            // Note: We can't use the helper methods here because the compiler can't reason
            // about the self.i64_values and self.entities being separable when hidden behind
            // the function calls.
            f(*id, i64_value);
        }
        Ok(())
    }
    fn for_each_operator<F: FnMut(Self::ID, &mut Operator)>(
        &mut self,
        f: &mut F,
    ) -> Result<(), Self::E> {
        for (id, operator) in &mut self.operators {
            // Note: We can't use the helper methods here because the compiler can't reason
            // about the self.operators and self.entities being separable when hidden behind
            // the function calls.
            f(*id, operator);
        }
        Ok(())
    }
    fn for_each_symbol<F: FnMut(Self::ID, &mut Symbol)>(
        &mut self,
        f: &mut F,
    ) -> Result<(), Self::E> {
        for (id, symbol) in &mut self.symbols {
            // Note: We can't use the helper methods here because the compiler can't reason
            // about the self.symbols and self.entities being separable when hidden behind
            // the function calls.
            f(*id, symbol);
        }
        Ok(())
    }
    fn for_each_call<F: FnMut(Self::ID, &mut Call<Self::ID>)>(
        &mut self,
        f: &mut F,
    ) -> Result<(), Self::E> {
        for (id, call) in &mut self.calls {
            // Note: We can't use the helper methods here because the compiler can't reason
            // about the self.calls and self.entities being separable when hidden behind
            // the function calls.
            f(*id, call);
        }
        Ok(())
    }
}

impl<T> NodeStore<EntityId, T, EcsError> for Ecs
where
    Self: Provider<T>,
{
    fn add(&mut self, value: T) -> EntityId {
        self.add_component(value)
    }

    fn overwrite(&mut self, id: EntityId, value: T) -> Result<Option<T>, EcsError> {
        //if let Ok(item) = self.get_mut(id) {
        //std::mem::swap(item, &mut value);
        //return Ok(Some(value));
        //}
        self.overwrite_entity(id, |_id| value)?;
        Ok(None) // The value didn't exist
    }

    fn get(&self, id: EntityId) -> Result<&T, EcsError> {
        <Ecs as Provider<T>>::get_component_for_entity(self, id)
    }
    #[allow(unused)]
    fn get_mut(&mut self, id: EntityId) -> Result<&mut T, EcsError>
    where
        Self: Provider<T>,
    {
        <Ecs as Provider<T>>::get_component_for_entity_mut(self, id)
    }

    fn remove(&mut self, id: EntityId) -> Result<Option<T>, EcsError> {
        Ok(Some(<Ecs as Provider<T>>::remove_component_for_entity(
            self, id,
        )?))
    }
}

impl Ecs {
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(test)]
    fn add<T>(&mut self, value: T) -> EntityId
    where
        Self: NodeStore<EntityId, T, EcsError>,
    {
        <Self as NodeStore<EntityId, T, EcsError>>::add(self, value)
    }

    #[cfg(test)]
    fn get<T>(&self, id: EntityId) -> Result<&T, EcsError>
    where
        Self: NodeStore<EntityId, T, EcsError>,
    {
        <Self as NodeStore<EntityId, T, EcsError>>::get(self, id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    type Call = super::Call<EntityId>;

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
            format!("Err(ComponentNotFound(\"steel::nodes::Call<steel::typed_index::TypedIndex<steel::ecs::component::Entity>>\", {:?}))", hello)
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
