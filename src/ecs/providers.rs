use super::Entity;
use crate::arena::ArenaError;
use crate::typed_index::TypedIndex;

pub type ComponentId<T> = TypedIndex<T>;

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

pub type EntityId = ComponentId<Entity>;

pub trait Provider<T> {
    type ID;
    fn add_with_id<F: FnOnce(EntityId) -> T>(&mut self, value: F) -> EntityId;
    fn overwrite_entity<F: FnOnce(EntityId) -> T>(
        &mut self,
        id: EntityId,
        value: F,
    ) -> Result<(), EcsError>;
    fn add_component(&mut self, value: T) -> EntityId {
        self.add_with_id(|_id| value)
    }
    fn get_component(&self, node: Self::ID) -> Result<&T, EcsError>;
    fn get_component_mut(&mut self, node: Self::ID) -> Result<&mut T, EcsError>;
    // fn remove_component(&mut self, node: Self::ID) -> Result<T, EcsError>;
    fn remove_component_for_entity(&mut self, id: EntityId) -> Result<T, EcsError>;
    fn get_component_for_entity(&self, id: EntityId) -> Result<&T, EcsError>;
    fn get_component_for_entity_mut(&mut self, id: EntityId) -> Result<&mut T, EcsError>;
}

#[cfg(test)]
mod test {}
