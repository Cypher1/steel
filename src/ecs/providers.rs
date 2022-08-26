use super::component::{ComponentID, ECSError, Entity};
use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use std::marker::PhantomData;

pub trait Provider<'a, T: 'a> {
    type ID;
    fn add_with_id<F: FnOnce(ID) -> T>(&mut self, value: F) -> ID; // Entity ID.
    fn add_component(&mut self, value: T) -> ID {
        self.add_with_id(|_id| value)
    }
    fn get_component(&self, node: Self::ID) -> Result<&T, ECSError>;
    fn get_component_mut(&mut self, node: Self::ID) -> Result<&mut T, ECSError>;
    fn get_component_for_entity(&self, id: ID) -> Result<&T, ECSError>;
    fn get_component_for_entity_mut(&mut self, id: ID) -> Result<&mut T, ECSError>;
}

#[cfg(test)]
mod test {}
