use crate::compiler_context::CompilerContext;
use crate::error::SteelErr;
use std::collections::HashMap;

#[derive(Debug)]
pub struct EvalState<T, ID> {
    pub function_stack: Vec<(ID, usize)>, // name -> memory address to store result.
    // Record all the bindings (i.e. name->index in memory stack).
    pub bindings: HashMap<String, Vec<usize>>, // name -> memory address to load result.
    pub mem_stack: Vec<T>, // results.
}

impl<T, ID> Default for EvalState<T, ID> {
    fn default() -> Self {
        Self {
            function_stack: Vec::new(),
            bindings: HashMap::new(),
            mem_stack: Vec::new(),
        }
    }
}
impl<T: Default + Clone, ID> EvalState<T, ID> {
    pub fn setup_call(&mut self, expr: ID) -> usize {
        let index = self.mem_stack.len();
        self.mem_stack.push(T::default()); // assume it's a 0...
        self.function_stack.push((expr, index)); // to evaluate...
        index
    }

    pub fn get_value_for(&mut self, name: &str) -> Option<T> {
        let bindings = self.bindings.get(name).cloned().unwrap_or_default();
        if let Some(binding) = bindings.last() {
            Some(self.mem_stack[*binding].clone())
        } else {
            None
        }
    }
}


pub fn eval<'context, C: CompilerContext>(context: &'context C, state: &mut EvalState<i64, C::ID>) -> Result<(), C::E>
    where
    <C as CompilerContext>::E: Into<SteelErr>, {
    while let Some((f, res_addr)) = state.function_stack.pop() {
        eprintln!("Step {:?}:\n  {}", f, context.pretty(f));
        step(context, state, f, res_addr)?;
    }
    Ok(())
}

pub fn step<'context, C: CompilerContext>(
    context: &'context C,
    state: &mut EvalState<i64, C::ID>,
    id: C::ID,
    res_index: usize,
) -> Result<(), C::E>
    where
    <C as CompilerContext>::E: Into<SteelErr>, {
    if let Ok(v) = context.get_i64(id) {
        state.mem_stack[res_index] = *v;
        return Ok(());
    }
    if let Ok(s) = context.get_symbol(id) {
        // if let Some(bound) = s.bound_to {
            // state.function_stack.push((bound, res_index));
            // return Ok(());
        // }
        if let Some(value) = state.get_value_for(&s.name) {
            state.mem_stack[res_index] = value;
            return Ok(());
        } else {
            match &*s.name {
                "*" => {
                    let l = state.get_value_for("arg_0");
                    let r = state.get_value_for("arg_1");
                    if let (Some(l), Some(r)) = (l, r) {
                        state.mem_stack[res_index] = l*r;
                        return Ok(());
                    } else {
                        todo!("Multiplication expects two arguments got {:?} {:?}", &l, &r);
                    }
                }
                "+" => {
                    let l = state.get_value_for("arg_0");
                    let r = state.get_value_for("arg_1");
                    if let (Some(l), Some(r)) = (l, r) {
                        state.mem_stack[res_index] = l+r;
                        return Ok(());
                    } else {
                        todo!("Addition expects two arguments got {:?} {:?}", &l, &r);
                    }
                }
                "/" => {
                    let l = state.get_value_for("arg_0");
                    let r = state.get_value_for("arg_1");
                    if let (Some(l), Some(r)) = (l, r) {
                        state.mem_stack[res_index] = l/r;
                        return Ok(());
                    } else {
                        todo!("Division expects two arguments got {:?} {:?}", &l, &r);
                    }
                }
                "-" => {
                    let l = state.get_value_for("arg_0");
                    let r = state.get_value_for("arg_1");
                    if let (Some(l), Some(r)) = (l, r) {
                        state.mem_stack[res_index] = l-r;
                        return Ok(());
                    } else {
                        todo!("Subtraction expects two arguments got {:?} {:?}", &l, &r);
                    }
                }
                _ => todo!("Unknown variable: {}", s.name),
            }
        }
    }
    if let Ok(c) = context.get_call(id) {
        // load in all the args
        let result = state.setup_call(c.callee);
        eprintln!("  inner {:?} -> {}", &result, context.pretty(c.callee));
        for (name, arg) in &c.args {
            eprintln!("    arg {:?} -> {}", &name, context.pretty(*arg));
            let index = state.setup_call(*arg);
            let entries = state.bindings.entry(name.clone()).or_insert_with(Vec::new);
            entries.push(index); // Vec allows shadowing
        }
        return Ok(());
    }
    // format!("{{node? {:?}}}", id)
    eprintln!("Unknown node {}, {:?}", context.pretty(id), id);
    todo!("Unknown node {:?}", id);
}
