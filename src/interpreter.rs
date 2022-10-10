use crate::compiler_context::CompilerContext;
use crate::error::SteelErr;
use log::{error, trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Imp<ID> = Arc<Mutex<dyn FnMut(&mut EvalState<ID>) -> Result<Value<ID>, SteelErr>>>;

#[derive(Clone)]
pub struct Impl<ID> {
    name: &'static str,
    imp: Imp<ID>,
}

impl<ID> Impl<ID> {
    fn new<F: 'static + FnMut(&mut EvalState<ID>) -> Result<Value<ID>, SteelErr>>(
        name: &'static str,
        imp: F,
    ) -> Self {
        Self {
            name,
            imp: Arc::new(Mutex::new(imp)),
        }
    }
}

impl<ID> std::fmt::Debug for Impl<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "'{}'", self.name)
    }
}

#[derive(Clone, Debug)]
pub enum Value<ID> {
    UnInit,
    I64(i64), // a raw i64 value.
    // TODO: Func(ID), // reference to an 'expression' like thing that can be evaluated in some context.
    Extern(Impl<ID>), // reference to an extern...
}

#[derive(Debug)]
pub enum FnPtr<ID> {
    StaticPtr(ID),
    MemPtr(usize), // Memory address
}
pub use FnPtr::*;

pub struct StackFrame<ID> {
    fn_ptr: FnPtr<ID>,
    return_address: usize,
    bindings: Vec<(String, usize)>,
}

fn state_to_string<C: CompilerContext>(
    context: &C,
    state: &EvalState<C::ID>,
    target: &StackFrame<C::ID>,
) -> String {
    let owning = if !target.bindings.is_empty() {
        format!("(owning {:?})", target.bindings)
    } else {
        "".to_string()
    };
    match target.fn_ptr {
        StaticPtr(ptr) => {
            format!("code {:?}{} -> {}", ptr, owning, context.pretty(ptr))
        }
        MemPtr(index) => {
            format!("value {:?}{} -> {:?}", index, owning, state.get_mem(index))
        }
    }
}

impl<ID: std::fmt::Debug> std::fmt::Debug for StackFrame<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "*{:?} = {:?}({:?})",
            self.return_address, self.fn_ptr, self.bindings
        )
    }
}

#[derive(Debug)]
pub struct EvalState<ID> {
    pub function_stack: Vec<StackFrame<ID>>, // name -> memory address to store result.
    // Record all the bindings (i.e. name->index in memory stack).
    pub bindings: HashMap<String, Vec<usize>>, // name -> memory address to load result.
    pub mem_stack: Vec<Value<ID>>,             // results.
}

impl<ID: std::fmt::Debug> EvalState<ID> {
    fn register_extern(mut self, imp: Impl<ID>) -> Self {
        // add binding
        let name = imp.name;
        let index = self.alloc(Value::Extern(imp));
        self.bind_name(name, index);
        self
    }
    fn run_extern(&mut self, imp: Impl<ID>) -> Result<Value<ID>, SteelErr> {
        // Get the Arc<Mutex<ImpFn>>
        let imp = imp.imp.clone();
        let mut imp = imp.lock().unwrap(); // Get the ImpFn.
        imp(self) // Run it
    }
}

impl<ID: Clone + std::fmt::Debug> Default for EvalState<ID> {
    fn default() -> Self {
        Self {
            function_stack: Vec::new(),
            bindings: HashMap::new(),
            mem_stack: Vec::new(),
        }
        .register_extern(Impl::new("+", |state| {
            bin_op(state, "+", |l, r| l.wrapping_add(r))
        }))
        .register_extern(Impl::new("-", |state| {
            bin_op(state, "-", |l, r| l.wrapping_sub(r))
        }))
        .register_extern(Impl::new("*", |state| {
            bin_op(state, "*", |l, r| l.wrapping_mul(r))
        }))
        .register_extern(Impl::new("/", |state| {
            bin_op(state, "/", |l, r| if r != 0 { l / r } else { 0 })
        }))
        .register_extern(Impl::new("putchar", |state: &mut EvalState<ID>| {
            if let Some(Value::I64(i)) = state.get_value_for("arg_0")? {
                if let Some(c) = char::from_u32(*i as u32) {
                    print!("{}", c);
                    return Ok(Value::I64(1));
                }
            }
            Ok(Value::I64(0)) // Could not print the unexpected value
        }))
    }
}
impl<ID> EvalState<ID> {
    fn set_mem(&mut self, index: usize, value: Value<ID>) {
        self.mem_stack[index] = value;
    }

    fn try_get_mem(&self, index: usize) -> Result<Option<&Value<ID>>, SteelErr> {
        let r = self.mem_stack.get(index);
        if let Some(Value::UnInit) = r {
            return Err(SteelErr::ReliedOnUnInitializedMemory(index));
        }
        Ok(r)
    }

    fn get_mem(&self, index: usize) -> Result<&Value<ID>, SteelErr> {
        self.try_get_mem(index)?
            .ok_or(SteelErr::ReliedOnOutOfBoundsMemory(index))
    }

    fn drop_mem(&mut self, mem: usize) {
        trace!("forgetting {:?} args", mem);
        let final_length = self.mem_stack.len().saturating_sub(mem);
        self.mem_stack.truncate(final_length);
    }

    fn alloc(&mut self, value: Value<ID>) -> usize {
        let index = self.mem_stack.len();
        self.mem_stack.push(value);
        index
    }

    pub fn bind_name(&mut self, name: &str, index: usize) {
        let entries = self
            .bindings
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        entries.push(index); // Vec allows shadowing
    }

    pub fn setup_eval_to(&mut self, fn_ptr: FnPtr<ID>, return_address: usize, bindings: Vec<(String, usize)>) {
        self.function_stack.push(StackFrame {
            fn_ptr,
            return_address,
            bindings,
        }); // to evaluate...
    }

    pub fn setup_closure(&mut self, code: ID, return_address: usize, mut bindings: Vec<(String, usize)>) -> usize {
        let callee_index = self.alloc(Value::UnInit); // explicitly store 'uninitialized' marker.
                                                      // then run the closure
        bindings.push(("self".to_string(), callee_index));
        self.setup_eval_to(
            FnPtr::MemPtr(callee_index),
            return_address,
            Vec::new(),
        );
        // but first fetch the 'code'.
        self.setup_eval_to(FnPtr::StaticPtr(code), callee_index, bindings);
        return_address
    }

    pub fn setup_eval(&mut self, target: FnPtr<ID>, bindings: Vec<(String, usize)>) -> usize {
        let return_address = self.alloc(Value::UnInit); // explicitly store 'uninitialized' marker.
        self.setup_eval_to(target, return_address, bindings);
        return_address
    }

    pub fn get_value_for(&mut self, name: &str) -> Result<Option<&Value<ID>>, SteelErr> {
        let mut bindings = self.bindings.get(name).cloned().unwrap_or_default();
        while let Some(binding) = bindings.last() {
            if let Some(value) = self.try_get_mem(*binding)? {
                return Ok(Some(value));
            }
            bindings.pop();
        }
        Ok(None)
    }
}

pub fn eval<C: CompilerContext>(context: &C, state: &mut EvalState<C::ID>) -> Result<(), SteelErr>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    while let Some(target) = state.function_stack.pop() {
        trace!("Evaluating {}", state_to_string(context, state, &target));
        step(context, state, target)?;
    }
    Ok(())
}

pub fn step<C: CompilerContext>(
    context: &C,
    state: &mut EvalState<C::ID>,
    target: StackFrame<C::ID>,
) -> Result<(), SteelErr>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    trace!("state: {:?}", state.mem_stack);
    perform(context, state, &target)?;
    // if target.bindings > 0 {
        // state.drop_mem(target.bindings);
    // }
    Ok(())
}

pub fn perform<C: CompilerContext>(
    context: &C,
    state: &mut EvalState<C::ID>,
    target: &StackFrame<C::ID>,
) -> Result<(), SteelErr>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    let StackFrame {
        fn_ptr,
        return_address,
        bindings,
    } = target;
    for (name, index) in bindings {
        state.bind_name(name, *index);
    }
    let id = match fn_ptr {
        MemPtr(index) => {
            let func = state.get_mem(*index)?.clone();
            // should drop the closure.
            trace!("running closure {:?} {:?}", func, target.bindings);
            let res = match func {
                Value::Extern(imp) => state.run_extern(imp)?,
                constant => constant,
            };
            state.set_mem(target.return_address, res);
            return Ok(()); // done!
        }
        StaticPtr(id) => *id,
    };
    if let Ok(c) = context.get_call(id) {
        // load in all the args
        let mut args = vec![];
        let mut todos = vec![];
        for (name, arg) in c.args.iter().rev() {
            trace!("    arg {:?} -> {}", &name, context.pretty(*arg));
            let index = state.alloc(Value::UnInit);
            args.push((name.to_string(), index));
            // TODO: Consider loading known values in without 'call'.
            todos.push((arg, index));
        }
        state.setup_closure(c.callee, *return_address, args);
        trace!(
            "  inner {:?} -> {}",
            &return_address,
            context.pretty(c.callee)
        );
        for (arg, index) in todos {
            state.setup_eval_to(FnPtr::StaticPtr(*arg), index, Vec::new());
        }
        return Ok(());
    }
    let res = if let Ok(v) = context.get_i64(id) {
        trace!("get constant i64 {}", &v);
        Value::I64(*v)
    } else if let Ok(s) = context.get_symbol(id) {
        trace!("get symbol {:?}", &s.name);
        state
            .get_value_for(&s.name)?
            .cloned()
            .ok_or_else(|| SteelErr::MissingValueForBinding(s.name.to_string()))?
    } else {
        // format!("{{node? {:?}}}", id)
        error!("Unknown node {}, {:?}", context.pretty(id), id);
        todo!("Unknown node {:?}", id);
    };
    state.set_mem(target.return_address, res);
    Ok(())
}

fn bin_op<ID: Clone + std::fmt::Debug, F: FnOnce(i64, i64) -> i64>(
    state: &mut EvalState<ID>,
    name: &str,
    op: F,
) -> Result<Value<ID>, SteelErr> {
    let l = state.get_value_for("arg_0")?.cloned();
    let l = if let Some(Value::I64(l)) = l {
        l
    } else {
        return Err(SteelErr::MissingArgumentExpectedByExtern(
            name.to_string(),
            "arg_0".to_string(),
        ));
    };
    let r = state.get_value_for("arg_1")?.cloned();
    let r = if let Some(Value::I64(r)) = r {
        r
    } else {
        return Err(SteelErr::MissingArgumentExpectedByExtern(
            name.to_string(),
            "arg_1".to_string(),
        ));
    };
    Ok(Value::I64(op(l, r)))
}
