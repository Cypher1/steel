use crate::compiler_context::CompilerContext;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

pub fn optimize<C: CompilerContext + ?Sized>(context: &mut C, id: C::ID) -> C::ID {
    loop {
        let fixed_point = AtomicBool::new(true);
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
        if fixed_point.load(Relaxed) {
            break;
        }
    }
    id
}
