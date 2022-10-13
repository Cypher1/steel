use crate::arena::{ArenaError, ID};
use crate::nodes::*;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum EcsError {
    InternalError(ArenaError),
    ComponentNotFound(ID),
}
use EcsError::*;

impl From<ArenaError> for EcsError {
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

#[derive(Default, Debug, Copy, Clone)]
pub struct Entity {
    pub symbol: Option<ComponentID<Symbol<ID>>>,
    pub call: Option<ComponentID<Call<ID>>>,
    pub i_64: Option<ComponentID<I64Value<ID>>>,
    pub optimizer_data: Option<ComponentID<OptimizerData<ID>>>,
}

#[cfg(test)]
mod test {}
