use crate::arena::{ArenaError, ID};
use std::marker::PhantomData;

impl From<ArenaError> for EcsError {
    fn from(it: ArenaError) -> Self {
        InternalError(it)
    }
}

#[derive(Debug, Clone)]
pub enum EcsError {
    InternalError(ArenaError),
    ComponentNotFound(ID),
}
use EcsError::*;

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


pub trait Provider<T> {
    type ID;
    fn add_with_id<F: FnOnce(ID) -> T>(&mut self, value: F) -> ID; // Entity ID.
    // fn replace_component(&mut self, id: ID, value: T) -> Result<(), EcsError>;
    // fn remove_component(&mut self, id: ID);
    fn add_component(&mut self, value: T) -> ID {
        self.add_with_id(|_id| value)
    }
    fn get_component(&self, node: Self::ID) -> Result<&T, EcsError>;
    fn get_component_mut(&mut self, node: Self::ID) -> Result<&mut T, EcsError>;
    fn get_component_for_entity(&self, id: ID) -> Result<&T, EcsError>;
    fn get_component_for_entity_mut(&mut self, id: ID) -> Result<&mut T, EcsError>;
}

#[cfg(test)]
mod test {}
