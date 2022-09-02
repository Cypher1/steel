use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use std::marker::PhantomData;

#[derive(Debug)]
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

#[derive(Debug, Copy, Clone)]
pub enum Entity {
    Symbol(ComponentID<Symbol<'static>>),
    Call(ComponentID<Call<ID>>),
    I64(ComponentID<i64>),
}

#[cfg(test)]
mod test {}
