use crate::compiler_context::CompilerContext;

pub fn optimize<C: CompilerContext + ?Sized>(context: &mut C, id: C::ID) -> C::ID {
    loop {
        let mut fixed_point = false;
        context.for_each(
            &|id, symbol| {

            },
            &|id, call| {

            },
            &|id, i64_value| {

            },
            &|id, optimizer_data| {

            },
        );
        if fixed_point {
            break;
        }
    }
    id
}
