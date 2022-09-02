use super::component::EcsError;
use crate::arena::ID;

pub trait Provider<'a, T: 'a> {
    type ID;
    fn add_with_id<F: FnOnce(ID) -> T>(&mut self, value: F) -> ID; // Entity ID.
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
