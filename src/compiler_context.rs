use crate::nodes::{Call, Symbol};

pub trait NodeStore<'source, ID, T, E> {
    fn add(&mut self, value: T) -> ID;
    fn get(&self, id: ID) -> Result<&T, E>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, E>;
}

pub trait CompilerContext<'source>:
    NodeStore<'source, Self::ID, Call<Self::ID>, Self::E>
    + NodeStore<'source, Self::ID, Symbol<'source>, Self::E>
    + NodeStore<'source, Self::ID, i64, Self::E>
{
    type ID: Copy + std::fmt::Debug;
    type E;

    fn new() -> Self;
    fn get_symbol(&self, id: Self::ID) -> Result<&Symbol<'source>, Self::E> {
        self.get(id)
    }
    fn get_symbol_mut(&mut self, id: Self::ID) -> Result<&mut Symbol<'source>, Self::E> {
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
        if let Ok(v) = self.get_i64(id) {
            return format!("{}", v);
        }
        if let Ok(s) = self.get_symbol(id) {
            return s.name.to_string();
        }
        if let Ok(c) = self.get_call(id) {
            let callee = self.pretty(c.callee);
            let is_operator_call = if let Ok(sym) = self.get_symbol(c.callee) {
                sym.is_operator
            } else {
                false
            };
            if is_operator_call {
                let args: Vec<String> = c.args.iter().map(|arg| self.pretty(*arg)).collect();
                let args = args.join(&callee);
                return format!("({}{})", if c.args.len() < 2 { &callee } else { "" }, args);
            }
            let args: Vec<String> = c.args.iter().map(|arg| self.pretty(*arg)).collect();
            let args = args.join(", ");
            return format!("{}({})", callee, args);
        }
        format!("{{node? {:?}}}", id)
    }
}

