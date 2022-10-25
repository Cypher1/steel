use crate::nodes::{Call, Operator, Symbol};

pub trait NodeStore<ID, T, E> {
    fn add(&mut self, value: T) -> ID;
    fn get(&self, id: ID) -> Result<&T, E>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, E>;
    fn remove(&mut self, id: ID) -> Result<Option<T>, E>;
    fn remove_any(&mut self, id: ID) {
        let _ = self.remove(id);
    }
    fn overwrite(&mut self, id: ID, mut value: T) -> Result<Option<T>, E> {
        let item = self.get_mut(id)?;
        std::mem::swap(item, &mut value);
        Ok(Some(value))
    }
}

pub type SysF<S, ID, T> = fn(&mut S, ID, &mut T);

pub trait CompilerContext:
    NodeStore<Self::ID, Call<Self::ID>, Self::E>
    + NodeStore<Self::ID, Symbol, Self::E>
    + NodeStore<Self::ID, Operator, Self::E>
    + NodeStore<Self::ID, i64, Self::E>
    + std::fmt::Debug
{
    type ID: Eq + std::hash::Hash + Copy + std::fmt::Debug;
    type E: Into<crate::error::SteelErr> + std::fmt::Debug;

    fn new() -> Self;
    fn get_operator(&self, id: Self::ID) -> Result<&Operator, Self::E> {
        self.get(id)
    }
    fn get_operator_mut(&mut self, id: Self::ID) -> Result<&mut Operator, Self::E> {
        self.get_mut(id)
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
    fn replace<T>(&mut self, id: Self::ID, value: T) -> Result<(), Self::E>
    where
        Self: NodeStore<Self::ID, T, Self::E>,
    {
        // For each component type...
        // TODO: What if we just ignored the 'dead' components???
        <Self as NodeStore<Self::ID, Call<Self::ID>, Self::E>>::remove_any(self, id);
        <Self as NodeStore<Self::ID, Symbol, Self::E>>::remove_any(self, id);
        <Self as NodeStore<Self::ID, Operator, Self::E>>::remove_any(self, id);
        <Self as NodeStore<Self::ID, i64, Self::E>>::remove_any(self, id);

        // TODO: Construct new, don't just get_mut...
        <Self as NodeStore<Self::ID, T, Self::E>>::overwrite(self, id, value).expect("FAILED!?");
        Ok(())
    }

    // Implement either all the `for_each_XXX`s or `for_each`
    // Call sites will pick whichever should work best for their use case.
    fn for_each_i64<F: FnMut(&mut Self, Self::ID, &mut i64)>(
        &mut self,
        f: &mut F,
    ) -> Result<(), Self::E> {
        self.for_each::<F, SysF<Self, Self::ID, _>, SysF<Self, Self::ID, _>, SysF<Self, Self::ID, _>>(
            &mut Some(f),
            &mut None,
            &mut None,
            &mut None,
        )
    }
    fn for_each_operator<F: FnMut(&mut Self, Self::ID, &mut Operator)>(
        &mut self,
        f: &mut F,
    ) -> Result<(), Self::E> {
        self.for_each::<SysF<Self, Self::ID, _>, F, SysF<Self, Self::ID, _>, SysF<Self, Self::ID, _>>(
            &mut None,
            &mut Some(f),
            &mut None,
            &mut None,
        )
    }
    fn for_each_symbol<F: FnMut(&mut Self, Self::ID, &mut Symbol)>(
        &mut self,
        f: &mut F,
    ) -> Result<(), Self::E> {
        self.for_each::<SysF<Self, Self::ID, _>, SysF<Self, Self::ID, _>, F, SysF<Self, Self::ID, _>>(
            &mut None,
            &mut None,
            &mut Some(f),
            &mut None,
        )
    }
    fn for_each_call<F: FnMut(&mut Self, Self::ID, &mut Call<Self::ID>)>(
        &mut self,
        f: &mut F,
    ) -> Result<(), Self::E> {
        self.for_each::<SysF<Self, Self::ID, _>, SysF<Self, Self::ID, _>, SysF<Self, Self::ID, _>, F>(
            &mut None,
            &mut None,
            &mut None,
            &mut Some(f),
        )
    }

    fn for_each<
        F1: FnMut(&mut Self, Self::ID, &mut i64),
        F2: FnMut(&mut Self, Self::ID, &mut Operator),
        F3: FnMut(&mut Self, Self::ID, &mut Symbol),
        F4: FnMut(&mut Self, Self::ID, &mut Call<Self::ID>),
    >(
        &mut self,
        i64_fn: &mut Option<&mut F1>,
        operator_fn: &mut Option<&mut F2>,
        symbol_fn: &mut Option<&mut F3>,
        call_fn: &mut Option<&mut F4>,
    ) -> Result<(), Self::E> {
        if let Some(i64_fn) = i64_fn {
            self.for_each_i64(i64_fn)?;
        }
        if let Some(operator_fn) = operator_fn {
            self.for_each_operator(operator_fn)?;
        }
        if let Some(symbol_fn) = symbol_fn {
            self.for_each_symbol(symbol_fn)?;
        }
        if let Some(call_fn) = call_fn {
            self.for_each_call(call_fn)?;
        }
        Ok(())
    }

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
