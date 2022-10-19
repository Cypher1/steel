use crate::arena::Index;
use std::marker::PhantomData;

#[derive(PartialEq, Eq, Hash)]
pub struct TypedIndex<T> {
    pub id: Index,
    pub ty: PhantomData<T>,
}

impl<T> std::fmt::Debug for TypedIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let full_name = std::any::type_name::<T>().to_string();
        let name = full_name.split("::").last().unwrap_or(&full_name);
        write!(f, "{} {}", name, self.id)
    }
}

impl<T> TypedIndex<T> {
    pub fn new(id: Index) -> Self {
        Self {
            id,
            ty: PhantomData::default(),
        }
    }
}

impl<T> Copy for TypedIndex<T> {}
impl<T> Clone for TypedIndex<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            ty: self.ty,
        }
    }
}

#[cfg(test)]
mod test {}

