use crate::compact_arena::Index;
use std::marker::PhantomData;

pub fn typed_descriptor<T>() -> String {
    let full_name = std::any::type_name::<T>().to_string();
    let type_name = full_name.split('<').next().unwrap_or(&full_name);
    let name = type_name.split("::").last().unwrap_or(&full_name);
    name.to_string()
}

#[derive(PartialEq, Eq, Hash)]
pub struct TypedIndex<T> {
    pub id: Index,
    pub ty: PhantomData<T>,
}

impl<T> std::fmt::Debug for TypedIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", typed_descriptor::<T>(), self.id)
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
