use crate::compiler_context::CompilerContext;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use log::{debug, trace};

#[derive(Default, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub struct Optimizations {
    constant_folding: bool,
}

type SharedMem<T> = Arc<Mutex<T>>;

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
    root: C::ID,
    fixed_point: &AtomicBool,
) -> Result<C::ID, C::E> {
    let known_values: SharedMem<HashMap<C::ID, i64>> = Arc::new(Mutex::new(HashMap::new()));
    let replace: SharedMem<Vec<(C::ID, i64)>> = Arc::new(Mutex::new(Vec::new()));
    let known_names: SharedMem<HashMap<C::ID, String>> = Arc::new(Mutex::new(HashMap::new()));
    let pass = "Constant folding";
    context.for_each(
        &|id, symbol, shared| {
            if shared.known_value_found {
                return;
            }
            shared.known_value_found = true;
            trace!("{}: {:?}", pass, &symbol);
            match &*symbol.name {
                "+" | "-" | "*" | "/" => {
                    // Just pretend that remapping operators is not possible...
                    let mut known_names = known_names.lock().unwrap();
                    known_names.insert(id, symbol.name.to_string());
                    fixed_point.store(false, Relaxed);
                }
                _ => {}
            }
        },
        &|id, call, shared| {
            if shared.known_value_found {
                trace!("{}: **DONE** {:?}", pass, call);
                return;
            }
            trace!("{}: {:?}", pass, call);
            let known_names = known_names.lock().unwrap();
            let name = if let Some(name) = known_names.get(&call.callee) {
                name
            } else {
                return; // noop now
            };
            trace!("{}: {:?}, {}", pass, call, name);
            let mut left: i64 = 0;
            let mut right: i64 = 0;
            for (arg_name, arg) in &call.args {
                let known_values = known_values.lock().unwrap();
                if let Some(value) = known_values.get(arg) {
                    if arg_name == "arg_0" {
                        left = *value;
                    } else if arg_name == "arg_1" {
                        right = *value;
                    }
                } else {
                    return;
                }
            }
            let result = match &**name {
                "+" => left.wrapping_add(right),
                "-" => left.wrapping_sub(right),
                "*" => left.wrapping_mul(right),
                "/" => left.wrapping_div(right),
                _ => {
                    todo!("HANDLE {} {}", pass, name);
                }
            };
            trace!(
                "{}: {} with {:?} {:?} gives {:?}",
                pass, name, left, right, result
            );
            shared.known_value_found = true;
            let mut known_values = known_values.lock().unwrap();
            known_values.insert(id, result);

            let mut replace = replace.lock().unwrap();
            replace.push((id, result));
            fixed_point.store(false, Relaxed);
            // i64_value.shared.optimizer_data.value = Some(i64_value.value);
        },
        &|id, i64_value, shared| {
            if shared.known_value_found {
                return;
            }
            trace!("{}: (i64) {}", pass, i64_value);
            shared.known_value_found = true;
            let mut known_values = known_values.lock().unwrap();
            known_values.insert(id, *i64_value);
            fixed_point.store(false, Relaxed);
        },
    )?;
    let replace = replace.lock().unwrap();
    for (id, value) in replace.iter() {
        debug!(
            "Replacing ent.{:?} (i.e. {}) with {:?}",
            id,
            context.pretty(*id),
            value
        );
        context.replace(*id, *value)?; // This is the bit that does the updates in place...
    }
    Ok(root)
}

pub fn optimize<C: CompilerContext + ?Sized>(
    context: &mut C,
    optimizations: &Optimizations,
    mut root: C::ID,
) -> Result<C::ID, C::E> {
    loop {
        let fixed_point = AtomicBool::new(true);
        if optimizations.constant_folding {
            root = constant_folding(context, root, &fixed_point)?;
        }
        if fixed_point.load(Relaxed) {
            break;
        }
    }
    Ok(root)
}
