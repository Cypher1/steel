use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum ECSError {
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
pub struct ComponentID<T> {
    pub id: ID,
    pub ty: PhantomData<T>,
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
pub enum Entity {
    Symbol(ComponentID<Symbol<'static>>),
    Call(ComponentID<Call<ID>>),
    I64(ComponentID<i64>),
}

pub trait Provider<'a, T: 'a> {
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

pub trait ArenaProvider<'a, T> {
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

#[macro_export]
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

#[cfg(test)]
mod test {
}
