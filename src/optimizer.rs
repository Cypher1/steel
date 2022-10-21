use crate::compiler_context::CompilerContext;
use crate::nodes::Operator;
// use log::{debug, trace};
use std::collections::HashMap;
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
    known_values: &mut HashMap<C::ID, i64>,
    operators: &mut HashMap<C::ID, Operator>,
    root: C::ID,
    fixed_point: &AtomicBool,
) -> Result<C::ID, C::E> {
    // Find nodes to replace
    // ECS will run the Call component, but AST has to traverse all the nodes to check if they
    // are Calls.
    {
        let ref operators = operators;
        let ref known_values = known_values;
        context.for_each_call(&mut|id, call| {
            let name = if let Some(name) = operators.get(&call.callee) {
                name
            } else {
                return; // skip now
            };
            let left: Option<i64> = call.left.map(|left| known_values.get(&left).map(|v|*v)).unwrap_or_default();
            let right: Option<i64> = call.right.map(|right| known_values.get(&right).map(|v|*v)).unwrap_or_default();
            use Operator::*;
            let result = match (name, left, right) {
                // Both known
                (Add, Some(left), Some(right)) => left.wrapping_add(right),
                (Sub, Some(left), Some(right)) => left.wrapping_sub(right),
                (Mul, Some(left), Some(right)) => left.wrapping_mul(right),
                (Div, Some(left), Some(right)) => left.wrapping_div(right),
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
                */
                _ => return,
            };
            // Update so that we don't have to re-find the updated values
            replace.push((id, result));
            fixed_point.store(false, Relaxed);
            // i64_value.shared.optimizer_data.value = Some(i64_value.value);
        })?;
    }
    for (id, value) in replace.iter() {
        known_values.insert(*id, *value);
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
    let mut known_values: HashMap<C::ID, i64> = HashMap::new();
    let mut operators: HashMap<C::ID, Operator> = HashMap::new();
    // Get all the known symbols and i64 values.
    // let pass = "Pre-pass for Constant folding";
    // ECS will run each component separately but
    // AST gets a benefit from running them during the same traversal.
    context.for_each(
        Some(&mut|id, i64_value| {
            known_values.insert(id, *i64_value);
        }),
        Some(&mut|id, operator| {
            operators.insert(id, *operator);
        }),
        None,
        None,
    )?;
    // Replace nodes
    let fixed_point = AtomicBool::new(true);
    let mut replace: Vec<(C::ID, i64)> = Vec::new();
    loop {
        fixed_point.store(true, Relaxed);
        if optimizations.constant_folding {
            root = constant_folding(context, &mut replace, &mut known_values, &mut operators, root, &fixed_point)?;
        }
        if fixed_point.load(Relaxed) {
            break;
        }
    }
    Ok(root)
}
