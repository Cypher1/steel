use crate::nodes::{Call, OptimizerData, Symbol};

pub trait NodeStore<ID, T, E> {
    fn add(&mut self, value: T) -> ID;
    fn get(&self, id: ID) -> Result<&T, E>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, E>;
}

pub trait CompilerContext:
    NodeStore<Self::ID, Call<Self::ID>, Self::E>
    + NodeStore<Self::ID, Symbol, Self::E>
    + NodeStore<Self::ID, OptimizerData<Self::ID>, Self::E>
    + NodeStore<Self::ID, i64, Self::E>
{
    type ID: Copy + std::fmt::Debug;
    type E: Into<crate::error::SteelErr>;

    fn new() -> Self;
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
    fn active_mem_usage(&self) -> usize;
    fn mem_usage(&self) -> usize;
    fn pretty(&self, id: Self::ID) -> String {
        use crate::pretty_printer::pretty;
        pretty(self, id)
    }
}
