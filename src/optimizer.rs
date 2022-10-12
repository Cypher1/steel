use crate::compiler_context::CompilerContext;

pub fn optimize<C: CompilerContext + ?Sized>(context: &mut C, id: C::ID) -> C::ID {
    loop {
        let mut fixed_point = false;
        context.for_each(
            &mut |id, symbol| {

            },
            &mut |id, call| {

            },
            &mut |id, i64_value| {

            },
            &mut |id, optimizer_data| {

            },
        );
        if fixed_point {
            break;
        }
    }
    id
}
