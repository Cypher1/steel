use crate::compiler_context::CompilerContext;
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
    _optimizations: &Optimizations,
    root: C::ID,
) -> Result<(bool, C::ID), C::E> {
    context.for_each(
        &|_id, symbol, shared| match &*symbol.name {
            "+" | "-" | "*" | "/" => {
                shared.optimizer_data.is_known_operation = Some(symbol.name.to_string());
            }
            _ => {}
        },
        &|_id, call, shared| {
            let _name = if let Some(name) = &shared.optimizer_data.is_known_operation {
                name
            } else {
                return;
            };
            for _arg in &call.args {}
            // i64_value.shared.optimizer_data.value = Some(i64_value.value);
        },
        &|_id, i64_value, shared| {
            shared.optimizer_data.is_known_value = Some(*i64_value);
        },
    )?;
    Ok((false, root))
}

pub fn optimize<C: CompilerContext + ?Sized>(
    context: &mut C,
    optimizations: &Optimizations,
    mut root: C::ID,
) -> Result<C::ID, C::E> {
    loop {
        let fixed_point = AtomicBool::new(true);
        if optimizations.constant_folding {
            let (changed, new_root) = constant_folding(context, optimizations, root)?;
            if changed {
                fixed_point.store(false, Relaxed);
                root = new_root;
            }
        }
        if fixed_point.load(Relaxed) {
            break;
        }
    }
    Ok(root)
}
