use crate::nodes::{Call, Shared, Symbol};

pub trait NodeStore<ID, T, E> {
    fn replace(&mut self, id: ID, value: T) -> Result<(), E>;
    fn add(&mut self, value: T) -> ID;
    fn get(&self, id: ID) -> Result<&T, E>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, E>;
}

pub type ForEachNode<'a, C, T> =
    &'a dyn Fn(<C as CompilerContext>::ID, &mut T, &mut Shared<<C as CompilerContext>::ID>);

pub trait CompilerContext:
    NodeStore<Self::ID, Call<Self::ID>, Self::E>
    + NodeStore<Self::ID, Symbol, Self::E>
    + NodeStore<Self::ID, i64, Self::E>
    + NodeStore<Self::ID, Shared<Self::ID>, Self::E>
{
    type ID: Eq + std::hash::Hash + Copy + std::fmt::Debug;
    type E: Into<crate::error::SteelErr> + std::fmt::Debug;

    fn new() -> Self;
    fn get_shared(&self, id: Self::ID) -> &Shared<Self::ID> {
        self.get(id).unwrap_or_else(|e| panic!("Missing shared on {:?}: {:?}", id, e))
    }
    fn get_shared_mut(&mut self, id: Self::ID) -> &mut Shared<Self::ID> {
        self.get_mut(id).unwrap_or_else(|e| panic!("Missing shared on {:?}: {:?}", id, e))
    }
    fn get_symbol(&self, id: Self::ID) -> Result<&Symbol, Self::E> {
        self.get(id)
    }
    fn get_symbol_mut(&mut self, id: Self::ID) -> Result<&mut Symbol, Self::E> {
        self.get_mut(id)
    }
    fn get_call(&self, id: Self::ID) -> Result<&Call<Self::ID>, Self::E> {
        self.get(id)
    }
    fn get_call_mut(&mut self, id: Self::ID) -> Result<&mut Call<Self::ID>, Self::E> {
        self.get_mut(id)
    }
    fn get_i64(&self, id: Self::ID) -> Result<&i64, Self::E> {
        self.get(id)
    }
    fn get_i64_mut(&mut self, id: Self::ID) -> Result<&mut i64, Self::E> {
        self.get_mut(id)
    }
    fn for_each(
        &mut self,
        symbol_fn: ForEachNode<Self, Symbol>,
        call_fn: ForEachNode<Self, Call<Self::ID>>,
        i64_fn: ForEachNode<Self, i64>,
    ) -> Result<(), Self::E>;
    fn active_mem_usage(&self) -> usize;
    fn mem_usage(&self) -> usize;
    fn pretty(&self, id: Self::ID) -> String {
        use crate::pretty_printer::pretty;
        pretty(self, id)
    }
    fn optimize(
        &mut self,
        optimizations: &crate::optimizer::Optimizations,
        id: Self::ID,
    ) -> Result<Self::ID, Self::E> {
        use crate::optimizer::optimize;
        optimize(self, optimizations, id)
    }
}
