use crate::compiler_context::CompilerContext;
use crate::nodes::Operator;
// use log::{debug, trace};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

#[derive(Default, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub struct Optimizations {
    constant_folding: bool,
}

impl Optimizations {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn and_constant_folding(self) -> Self {
        Self {
            constant_folding: true,
            ..self
        }
    }

    pub fn all(self) -> Self {
        self.and_constant_folding()
    }
}

fn constant_folding<C: CompilerContext + ?Sized>(
    context: &mut C,
    replace: &mut Vec<(C::ID, i64)>,
    root: C::ID,
    fixed_point: &AtomicBool,
) -> Result<C::ID, C::E> {
    // Find nodes to replace
    // ECS will run the Call component, but AST has to traverse all the nodes to check if they
    // are Calls.
    context.for_each_call(&mut |context, id, call| {
        let name = if let Ok(name) = context.get_operator(call.callee) {
            name
        } else {
            return; // skip now
        };
        let left: i64 = if let Some(left) = call.left {
            if let Ok(left) = context.get_i64(left) {
                *left
            } else {
                return;
            }
        } else {
            return;
        };
        let right: i64 = if let Some(right) = call.right {
            if let Ok(right) = context.get_i64(right) {
                *right
            } else {
                return;
            }
        } else {
            return;
        };
        use Operator::*;
        let result = match (name, left, right) {
            // Both known
            (Add, left, right) => left.wrapping_add(right),
            (Sub, left, right) => left.wrapping_sub(right),
            (Mul, left, right) => left.wrapping_mul(right),
            (Div, left, right) => left.wrapping_div(right),
            // One known (other discarded): Only sound if no side-effects...
            // (Mul, Some(0), _) | (Mul, _, Some(0)) => 0,
            // (Div, Some(0), _) => 0,
            // (Div, _, Some(0)) => 0, // TODO: Error values (/0)
            // One known (reduces to remaining)
            /*
            (Add, Some(0), Some(_zero_id), _, Some(value_id)) | (Add, _, Some(value_id), Some(0), Some(_zero_id)) => todo!(), // value_id
            (Sub, Some(0), Some(_zero_id), _, Some(value_id)) => todo!(), // -*value_id
            (Sub, _, Some(value_id), Some(0), Some(_zero_id)) => todo!(), // value_id
            (Mul, Some(1), Some(_zero_id), _, Some(value_id)) | (Mul, _, Some(value_id), Some(1), Some(_zero_id)) => todo!(), // value_id
            (Mul, _, Some(value_id), Some(1), Some(_zero_id)) => todo!(), // value_id
            _ => return,
            */
        };
        // Update so that we don't have to re-find the updated values
        replace.push((id, result));
        fixed_point.store(false, Relaxed);
        // i64_value.shared.optimizer_data.value = Some(i64_value.value);
    })?;
    for (id, value) in replace.iter() {
        // println!("REPLACING {:?} with value: {}", id, value);
        context.replace(*id, *value)?; // This is the bit that does the updates in place...
    }
    replace.clear(); // no need to replace nodes twice (but keep the capacity for later).
    Ok(root)
}

pub fn optimize<C: CompilerContext + ?Sized>(
    context: &mut C,
    optimizations: &Optimizations,
    mut root: C::ID,
) -> Result<C::ID, C::E> {
    // Replace nodes
    let fixed_point = AtomicBool::new(true);
    let mut replace: Vec<(C::ID, i64)> = Vec::new();
    loop {
        fixed_point.store(true, Relaxed);
        if optimizations.constant_folding {
            root = constant_folding(
                context,
                &mut replace,
                root,
                &fixed_point,
            )?;
        }
        if fixed_point.load(Relaxed) {
            break;
        }
    }
    Ok(root)
}
