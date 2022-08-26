use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use std::marker::PhantomData;

#[derive(Debug)]
enum ECSError {
    InternalError(ArenaError),
    ComponentNotFound(ID),
}
use ECSError::*;

impl From<ArenaError> for ECSError {
    fn from(it: ArenaError) -> Self {
        InternalError(it)
    }
}

#[derive(Debug)]
struct ComponentID<T> {
    id: ID,
    ty: PhantomData<T>,
}

impl<T> Copy for ComponentID<T> {}
impl<T> Clone for ComponentID<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            ty: self.ty,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Entity {
    Symbol(ComponentID<Symbol<'static>>),
    Call(ComponentID<Call<ID>>),
    I64(ComponentID<i64>),
}

#[derive(Debug, Default)]
struct Context<'source> {
    entities: Arena<Entity>,
    symbols: Arena<Symbol<'source>>,
    calls: Arena<Call<ID>>,
    int64_values: Arena<i64>,
}

impl<'source> Context<'source> {
    fn get<T: 'source>(&self, id: ID) -> Result<&T, ECSError>
    where
        Self: Provider<'source, T>,
    {
        <Context<'source> as Provider<'source, T>>::get(self, id)
    }
    #[allow(unused)]
    fn get_mut<T: 'source>(&mut self, id: ID) -> Result<&mut T, ECSError>
    where
        Self: Provider<'source, T>,
    {
        <Context<'source> as Provider<'source, T>>::get_mut(self, id)
    }
}

trait Provider<'a, T: 'a> {
    type ID;
    fn add_with_id<F: FnOnce(ID) -> T>(&mut self, value: F) -> ID; // Entity ID.
    fn add(&mut self, value: T) -> ID {
        self.add_with_id(|_id| value)
    }
    fn get_component(&self, node: Self::ID) -> Result<&T, ECSError>;
    fn get_component_mut(&mut self, node: Self::ID) -> Result<&mut T, ECSError>;

    fn get(&self, id: ID) -> Result<&T, ECSError>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, ECSError>;
}

trait ArenaProvider<'a, T> {
    fn entities(&self) -> &Arena<Entity>;
    fn entities_mut(&mut self) -> &mut Arena<Entity>;
    fn make_entity(id: ID) -> Entity;
    fn arena(&self) -> (&Arena<Entity>, &Arena<T>);
    fn arena_mut(&mut self) -> (&mut Arena<Entity>, &mut Arena<T>);
    fn get_impl(&self, id: ID) -> Result<&T, ECSError>;
    fn get_mut_impl(&mut self, id: ID) -> Result<&mut T, ECSError>;
}

impl<'a, T: 'a, S: ArenaProvider<'a, T>> Provider<'a, T> for S {
    type ID = ComponentID<T>;
    fn add_with_id<F: FnOnce(ID) -> T>(&mut self, value: F) -> ID {
        let (entities, arena) = self.arena_mut();
        entities.add_with_id(|id| {
            let node = arena.add(value(id)); // raw id and raw component id.
            Self::make_entity(node)
        })
    }
    fn get_component(&self, id: Self::ID) -> Result<&T, ECSError> {
        Ok(self.arena().1.get(id.id)?)
    }
    fn get_component_mut(&mut self, id: Self::ID) -> Result<&mut T, ECSError> {
        Ok(self.arena_mut().1.get_mut(id.id)?)
    }
    fn get(&self, id: ID) -> Result<&T, ECSError> {
        self.get_impl(id)
    }
    fn get_mut(&mut self, id: ID) -> Result<&mut T, ECSError> {
        self.get_mut_impl(id)
    }
}

macro_rules! make_provider {
    ($ctx: ty, $type: ty, $kind: tt, $accessor: tt) => {
        impl<'a> ArenaProvider<'a, $type> for $ctx {
            fn entities(&self) -> &Arena<Entity> {
                &self.entities
            }
            fn entities_mut(&mut self) -> &mut Arena<Entity> {
                &mut self.entities
            }
            fn make_entity(id: ID) -> Entity {
                Entity::$kind(ComponentID {
                    id,
                    ty: PhantomData,
                })
            }
            fn arena(&self) -> (&Arena<Entity>, &Arena<$type>) {
                (&self.entities, &self.$accessor)
            }
            fn arena_mut(&mut self) -> (&mut Arena<Entity>, &mut Arena<$type>) {
                (&mut self.entities, &mut self.$accessor)
            }
            fn get_impl(&self, id: ID) -> Result<&$type, ECSError> {
                let ent = self.entities.get(id)?;
                match ent {
                    Entity::$kind(component_id) => Ok(self.get_component(*component_id)?),
                    _ => Err(ECSError::ComponentNotFound(id)),
                }
            }
            fn get_mut_impl(&mut self, id: ID) -> Result<&mut $type, ECSError> {
                let ent = self.entities.get(id)?;
                match ent {
                    Entity::$kind(component_id) => Ok(self.get_component_mut(*component_id)?),
                    _ => Err(ECSError::ComponentNotFound(id)),
                }
            }
        }
    };
}

make_provider!(Context<'a>, Symbol<'a>, Symbol, symbols);
make_provider!(Context<'a>, Call<ID>, Call, calls);
make_provider!(Context<'a>, i64, I64, int64_values);

impl<'source> Context<'source> {
    fn new() -> Self {
        Default::default()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    type Call = super::Call<ID>;

    #[test]
    fn can_construct_node() -> Result<(), ECSError> {
        let mut ctx: Context<'static> = Context::new();

        let hello = ctx.add(Symbol::new("hello"));

        let sym: &Symbol<'static> = ctx.get(hello)?;
        assert_eq!(format!("{:?}", sym), "Symbol { name: \"hello\" }");
        Ok(())
    }

    #[test]
    fn cannot_access_incorrect_node() {
        let mut ctx: Context<'static> = Context::new();
        let hello = ctx.add(Symbol::new("hello"));
        let call: Result<&Call, ECSError> = ctx.get(hello);
        assert_eq!(
            format!("{:?}", call),
            format!("Err(ComponentNotFound({:?}))", hello)
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Context<'static> = Context::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get::<Symbol<'static>>(hello)),
            "Ok(Symbol { name: \"hello\" })"
        );
        assert_eq!(
            format!("{:?}", ctx.get::<Symbol<'static>>(world)),
            "Ok(Symbol { name: \"world\" })"
        );
    }

    #[test]
    fn can_construct_nodes_with_self_reference() {
        let mut ctx: Context<'static> = Context::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [] }})", reference)
        );
    }

    #[test]
    fn can_construct_nodes_with_cross_reference() {
        let mut ctx: Context<'static> = Context::new();

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
        let mut ctx: Context<'static> = Context::new();

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
