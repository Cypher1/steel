use crate::arena::{ArenaError, Index};
use std::marker::PhantomData;
use super::Entity;

impl From<ArenaError> for EcsError {
    fn from(it: ArenaError) -> Self {
        InternalError(it)
    }
}

#[derive(Debug, Clone)]
pub enum EcsError {
    InternalError(ArenaError),
    ComponentNotFound(String, EntityId),
}
use EcsError::*;

#[derive(PartialEq, Eq, Hash)]
pub struct ComponentId<T> {
    pub id: Index,
    pub ty: PhantomData<T>,
}

impl<T> std::fmt::Debug for ComponentId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", std::any::type_name::<T>(), self.id)
    }
}

impl<T> ComponentId<T> {
    pub fn new(id: Index) -> Self {
        Self {
            id,
            ty: PhantomData::default(),
        }
    }
}

impl<T> Copy for ComponentId<T> {}
impl<T> Clone for ComponentId<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            ty: self.ty,
        }
    }
}

pub type EntityId = ComponentId<Entity>;

pub trait Provider<T> {
    type ID;
    fn add_with_id<F: FnOnce(EntityId) -> T>(&mut self, value: F) -> EntityId; // Entity ID.
    fn add_component(&mut self, value: T) -> EntityId {
        self.add_with_id(|_id| value)
    }
    fn get_component(&self, node: Self::ID) -> Result<&T, EcsError>;
    fn get_component_mut(&mut self, node: Self::ID) -> Result<&mut T, EcsError>;
    fn remove_component(&mut self, node: Self::ID) -> Result<T, EcsError>;
    fn remove_component_for_entity(&mut self, id: EntityId) -> Result<T, EcsError>;
    fn get_component_for_entity(&self, id: EntityId) -> Result<&T, EcsError>;
    fn get_component_for_entity_mut(&mut self, id: EntityId) -> Result<&mut T, EcsError>;
}

#[cfg(test)]
mod test {}
