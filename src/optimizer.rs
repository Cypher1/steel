use crate::compiler_context::CompilerContext;
use crate::nodes::Operator;
// use log::{debug, trace};

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

fn constant_folding<C: CompilerContext + ?Sized + std::fmt::Debug>(
    context: &mut C,
    replace: &mut Vec<(C::ID, i64)>,
    root: C::ID,
    fixed_point: &mut bool,
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
        let result = match name {
            Add => left.wrapping_add(right),
            Sub => left.wrapping_sub(right),
            Mul => left.wrapping_mul(right),
            Div => left.wrapping_div(right),
        };
        // Update so that we don't have to re-find the updated values
        replace.push((id, result));
        *fixed_point = false;
    })?;
    for (id, value) in replace.iter() {
        // println!("REPLACING {:?} with value: {}", id, value);
        context.replace(*id, *value)?; // This is the bit that does the updates in place...
    }
    replace.clear(); // no need to replace nodes twice (but keep the capacity for later).
    Ok(root)
}

pub fn optimize<C: CompilerContext + ?Sized + std::fmt::Debug>(
    context: &mut C,
    optimizations: &Optimizations,
    mut root: C::ID,
) -> Result<C::ID, C::E> {
    // Replace nodes
    let mut fixed_point;
    let mut replace: Vec<(C::ID, i64)> = Vec::new();
    loop {
        fixed_point = true;
        if optimizations.constant_folding {
            root = constant_folding(
                context,
                &mut replace,
                root,
                &mut fixed_point,
            )?;
        }
        if fixed_point {
            break;
        }
    }
    Ok(root)
}
