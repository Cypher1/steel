use crate::arena::{Arena, ArenaError, ID};
use crate::nodes::*;
use std::marker::PhantomData;

#[macro_use]
mod component;
use component::*;

#[derive(Debug, Default)]
struct Context<'source> {
    entities: Arena<Entity>,
    symbols: Arena<Symbol<'source>>,
    calls: Arena<Call<ID>>,
    int64_values: Arena<i64>,
}

impl<'source> Context<'source> {
    fn get<T: 'source>(&self, id: ID) -> Result<&T, ECSError>
    where
        Self: Provider<'source, T>,
    {
        <Context<'source> as Provider<'source, T>>::get(self, id)
    }
    #[allow(unused)]
    fn get_mut<T: 'source>(&mut self, id: ID) -> Result<&mut T, ECSError>
    where
        Self: Provider<'source, T>,
    {
        <Context<'source> as Provider<'source, T>>::get_mut(self, id)
    }
}

make_provider!(Context<'a>, Symbol<'a>, Symbol, symbols);
make_provider!(Context<'a>, Call<ID>, Call, calls);
make_provider!(Context<'a>, i64, I64, int64_values);

impl<'source> Context<'source> {
    fn new() -> Self {
        Default::default()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    type Call = super::Call<ID>;

    #[test]
    fn can_construct_node() -> Result<(), ECSError> {
        let mut ctx: Context<'static> = Context::new();

        let hello = ctx.add(Symbol::new("hello"));

        let sym: &Symbol<'static> = ctx.get(hello)?;
        assert_eq!(format!("{:?}", sym), "Symbol { name: \"hello\" }");
        Ok(())
    }

    #[test]
    fn cannot_access_incorrect_node() {
        let mut ctx: Context<'static> = Context::new();
        let hello = ctx.add(Symbol::new("hello"));
        let call: Result<&Call, ECSError> = ctx.get(hello);
        assert_eq!(
            format!("{:?}", call),
            format!("Err(ComponentNotFound({:?}))", hello)
        );
    }

    #[test]
    fn can_construct_nodes() {
        let mut ctx: Context<'static> = Context::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));

        assert_eq!(
            format!("{:?}", ctx.get::<Symbol<'static>>(hello)),
            "Ok(Symbol { name: \"hello\" })"
        );
        assert_eq!(
            format!("{:?}", ctx.get::<Symbol<'static>>(world)),
            "Ok(Symbol { name: \"world\" })"
        );
    }

    #[test]
    fn can_construct_nodes_with_self_reference() {
        let mut ctx: Context<'static> = Context::new();

        let reference = ctx.add_with_id(|id| Call::new(id, vec![]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [] }})", reference)
        );
    }

    #[test]
    fn can_construct_nodes_with_cross_reference() {
        let mut ctx: Context<'static> = Context::new();

        let hello = ctx.add(Symbol::new("hello"));
        let world = ctx.add(Symbol::new("world"));
        let reference = ctx.add(Call::new(hello, vec![world]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!("Ok(Call {{ callee: {:?}, args: [{:?}] }})", hello, world)
        );
    }

    #[test]
    fn can_construct_values() {
        let mut ctx: Context<'static> = Context::new();

        let plus = ctx.add(Symbol::new("plus"));
        let a = ctx.add(32i64);
        let b = ctx.add(12i64);
        let reference = ctx.add(Call::new(plus, vec![a, b]));

        assert_eq!(
            format!("{:?}", ctx.get::<Call>(reference)),
            format!(
                "Ok(Call {{ callee: {:?}, args: [{:?}, {:?}] }})",
                plus, a, b
            )
        );
    }
}
