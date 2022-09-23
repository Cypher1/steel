use crate::nodes::{Call, Symbol};

#[derive(Debug)]
pub enum EvalResult<T, ID> {
    Value(T),
    Step(ID),
}
use EvalResult::*;

#[derive(Debug)]
pub struct EvalState<T, ID> {
    pub function_stack: Vec<ID>,
    pub mem_stack: Vec<T>,
}

impl<T, ID> Default for EvalState<T, ID> {
    fn default() -> Self {
        Self {
            function_stack: Vec::new(),
            mem_stack: Vec::new(),
        }
    }
}

pub trait NodeStore<ID, T, E> {
    fn add(&mut self, value: T) -> ID;
    fn get(&self, id: ID) -> Result<&T, E>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, E>;
}

pub trait CompilerContext:
    NodeStore<Self::ID, Call<Self::ID>, Self::E>
    + NodeStore<Self::ID, Symbol<Self::ID>, Self::E>
    + NodeStore<Self::ID, i64, Self::E>
{
    type ID: Copy + std::fmt::Debug;
    type E;

    fn new() -> Self;
    fn get_symbol(&self, id: Self::ID) -> Result<&Symbol<Self::ID>, Self::E> {
        self.get(id)
    }
    fn get_symbol_mut(&mut self, id: Self::ID) -> Result<&mut Symbol<Self::ID>, Self::E> {
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
    fn eval(&self, state: &mut EvalState<i64, Self::ID>) -> Result<(), Self::E> {
        while let Some(f) = state.function_stack.pop() {
            let result = self.step(state, f)?;
            match result {
                Step(f) => state.function_stack.push(f),
                Value(v) => state.mem_stack.push(v),
            }
        }
        Ok(())
    }

    fn step(
        &self,
        _state: &mut EvalState<i64, Self::ID>,
        id: Self::ID,
    ) -> Result<EvalResult<i64, Self::ID>, Self::E> {
        if let Ok(v) = self.get_i64(id) {
            return Ok(Value(*v));
        }
        if let Ok(s) = self.get_symbol(id) {
            if let Some(bound) = s.bound_to {
                return Ok(Step(bound));
            }
            todo!("Unknown variable: {}", s.name);
        }
        /*
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
        */
        todo!("Unknown node {:?}", id);
    }
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
            let mut arg_num = 0;
            if is_operator_call {
                let args: Vec<String> = c
                    .args
                    .iter()
                    .map(|(name, arg)| {
                        if name.starts_with("arg_") && name == &format!("arg_{}", arg_num) {
                            arg_num += 1;
                            self.pretty(*arg)
                        } else {
                            format!("{}={}", name, self.pretty(*arg))
                        }
                    })
                    .collect();
                let args = args.join(&callee);
                return format!("({}{})", if c.args.len() < 2 { &callee } else { "" }, args);
            }
            let args: Vec<String> = c
                .args
                .iter()
                .map(|(name, arg)| {
                    if name.starts_with("arg_") && name == &format!("arg_{}", arg_num) {
                        arg_num += 1;
                        self.pretty(*arg)
                    } else {
                        format!("{}={}", name, self.pretty(*arg))
                    }
                })
                .collect();
            let args = args.join(", ");
            return format!("{}({})", callee, args);
        }
        format!("{{node? {:?}}}", id)
    }
}
