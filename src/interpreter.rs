use crate::compiler_context::CompilerContext;
use crate::error::SteelErr;
use std::collections::HashMap;
use log::{trace, error};

#[derive(Debug)]
pub struct EvalState<T, ID> {
    pub function_stack: Vec<(ID, usize, usize)>, // name -> memory address to store result.
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
    pub fn setup_call_to(&mut self, expr: ID, res:usize, args: usize) -> usize {
        self.function_stack.push((expr, res, args)); // to evaluate...
        res
    }

    pub fn setup_call(&mut self, expr: ID, args: usize) -> usize {
        let index = self.mem_stack.len();
        self.mem_stack.push(T::default()); // assume it's a 0...
        self.setup_call_to(expr, index, args)
    }

    pub fn get_value_for(&mut self, name: &str) -> Option<T> {
        let mut bindings = self.bindings.get(name).cloned().unwrap_or_default();
        while let Some(binding) = bindings.last() {
            if *binding < self.mem_stack.len() {
                return Some(self.mem_stack[*binding].clone());
            }
            bindings.pop();
        }
        None
    }
}


pub fn eval<C: CompilerContext>(context: &C, state: &mut EvalState<i64, C::ID>) -> Result<(), C::E>
    where
    <C as CompilerContext>::E: Into<SteelErr>, {
    while let Some((f, res_addr, args)) = state.function_stack.pop() {
        trace!("Step {:?}:", f);
        trace!("{}", context.pretty(f));
        step(context, state, f, res_addr, args)?;
    }
    Ok(())
}

pub fn step<C: CompilerContext>(
    context: &C,
    state: &mut EvalState<i64, C::ID>,
    id: C::ID,
    res_index: usize,
    args: usize // number of args to pop
) -> Result<(), C::E>
    where
    <C as CompilerContext>::E: Into<SteelErr>, {
    if let Some(s) = perform(context, state, id, res_index, args)? {
        state.mem_stack[res_index] = s;
        if args > 0 {
            trace!("Forgetting {:?} args", args);
            let final_length = state.mem_stack.len().saturating_sub(args);
            state.mem_stack.truncate(final_length);
        }
    }
    Ok(())
}

fn un_op<C: CompilerContext, F: FnOnce(i64)->i64>(
    _context: &C,
    state: &mut EvalState<i64, C::ID>,
    name: &str,
    op: F) -> i64 {
    let l = state.get_value_for("arg_0");
    if let Some(l) = l {
        op(l)
    } else {
        todo!("{} expects one argument got {:?}", name, &l);
    }
}

fn bin_op<C: CompilerContext, F: FnOnce(i64, i64)->i64>(
    _context: &C,
    state: &mut EvalState<i64, C::ID>,
    name: &str,
    op: F) -> i64 {
    let l = state.get_value_for("arg_0");
    let r = state.get_value_for("arg_1");
    if let (Some(l), Some(r)) = (l, r) {
        op(l,r)
    } else {
        todo!("{} expects two arguments got {:?} {:?}", name, &l, &r);
    }
}

pub fn perform<C: CompilerContext>(
    context: &C,
    state: &mut EvalState<i64, C::ID>,
    id: C::ID,
    res_index: usize,
    args: usize // number of args to pop
) -> Result<Option<i64>, C::E>
    where
    <C as CompilerContext>::E: Into<SteelErr>, {
    if let Ok(v) = context.get_i64(id) {
        return Ok(Some(*v));
    }
    if let Ok(s) = context.get_symbol(id) {
        // if let Some(bound) = s.bound_to {
            // state.function_stack.push((bound, res_index));
            // Ok(())
        // }
        let r = if let Some(value) = state.get_value_for(&s.name) {
            value
        } else {
            match &*s.name {
                "putchar" => un_op(context, state, "Putchar", |i|{
                    if let Some(c) = char::from_u32(i as u32) {
                        print!("{}", c);
                        1
                    } else {
                        0
                    }
                }),
                "+" => bin_op(context, state, "Addition", |l, r|l+r),
                "-" => bin_op(context, state, "Subtraction", |l, r|l-r),
                "*" => bin_op(context, state, "Multiplication", |l, r|l*r),
                "/" => bin_op(context, state, "Division", |l, r|l/r),
                _ => todo!("Unknown variable: {}", s.name),
            }
        };
        return Ok(Some(r));
    }
    if let Ok(c) = context.get_call(id) {
        // load in all the args
        let result = state.setup_call_to(c.callee, res_index, args+c.args.len());
        trace!("  inner {:?} -> {}", &result, context.pretty(c.callee));
        for (name, arg) in c.args.iter().rev() {
            trace!("    arg {:?} -> {}", &name, context.pretty(*arg));
            let index = state.setup_call(*arg, 0);
            let entries = state.bindings.entry(name.clone()).or_insert_with(Vec::new);
            entries.push(index); // Vec allows shadowing
        }
        return Ok(None);
    }
    // format!("{{node? {:?}}}", id)
    error!("Unknown node {}, {:?}", context.pretty(id), id);
    todo!("Unknown node {:?}", id);
}
