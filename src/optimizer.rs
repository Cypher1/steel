use crate::compiler_context::CompilerContext;
use log::{debug, trace};
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
    known_values: &mut HashMap<C::ID, i64>,
    known_names: &mut HashMap<C::ID, String>,
    root: C::ID,
    fixed_point: &AtomicBool,
) -> Result<C::ID, C::E> {
    let mut replace: Vec<(C::ID, i64)> = Vec::new();
    let pass = "Constant folding";
    // Find nodes to replace
    // ECS will run the Call component, but AST has to traverse all the nodes to check if they
    // are Calls.
    {
        let ref mut replace = replace;
        let ref known_names = known_names;
        let ref known_values = known_values;
        context.for_each_call(&mut|id, call| {
            trace!("{}: {:?}", pass, call);
            let name = if let Some(name) = known_names.get(&call.callee) {
                name
            } else {
                return; // skip now
            };
            trace!("{}: {:?}, {}", pass, call, name);
            let mut left: Option<i64> = None;
            let mut right: Option<i64> = None;
            for (arg_name, arg) in &call.args {
                if arg_name == "arg_0" {
                    left = known_values.get(arg).cloned();
                } else if arg_name == "arg_1" {
                    right = known_values.get(arg).cloned();
                }
            }
            let result = match (&**name, left, right) {
                ("+", Some(left), Some(right)) => left.wrapping_add(right),
                ("-", Some(left), Some(right)) => left.wrapping_sub(right),
                ("*", Some(left), Some(right)) => left.wrapping_mul(right),
                ("/", Some(left), Some(right)) => left.wrapping_div(right),
                _ => return,
            };
            trace!(
                "{}: {} with {:?} {:?} gives {:?}",
                pass,
                name,
                left,
                right,
                result
            );
            // Update so that we don't have to re-find the updated values
            replace.push((id, result));
            fixed_point.store(false, Relaxed);
            // i64_value.shared.optimizer_data.value = Some(i64_value.value);
        })?;
    }
    for (id, value) in replace.iter() {
        debug!(
            "Replacing ent.{:?} (i.e. {}) with {:?}",
            id,
            context.pretty(*id),
            value
        );
        known_values.insert(*id, *value);
        context.replace(*id, *value)?; // This is the bit that does the updates in place...
    }
    Ok(root)
}

pub fn optimize<C: CompilerContext + ?Sized>(
    context: &mut C,
    optimizations: &Optimizations,
    mut root: C::ID,
) -> Result<C::ID, C::E> {
    let mut known_values: HashMap<C::ID, i64> = HashMap::new();
    let mut known_names: HashMap<C::ID, String> = HashMap::new();
    // Get all the known symbols and i64 values.
    // let pass = "Pre-pass for Constant folding";
    // ECS will run each component separately but
    // AST gets a benefit from running them during the same traversal.
    context.for_each(
        Some(&mut|id, symbol| {
            // trace!("{}: {:?}", pass, &symbol);
            match &*symbol.name {
                "+" | "-" | "*" | "/" => {
                    // Just pretend that remapping operators is not possible...
                    known_names.insert(id, symbol.name.to_string());
                }
                _ => {}
            }
        }),
        None,
        Some(&mut|id, i64_value| {
            // trace!("{}: (i64) {}", pass, i64_value);
            known_values.insert(id, *i64_value);
        }),
    )?;
    // Replace nodes
    let fixed_point = AtomicBool::new(true);
    loop {
        fixed_point.store(true, Relaxed);
        if optimizations.constant_folding {
            root = constant_folding(context, &mut known_values, &mut known_names, root, &fixed_point)?;
        }
        if fixed_point.load(Relaxed) {
            break;
        }
    }
    Ok(root)
}
