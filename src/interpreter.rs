use crate::compiler_context::CompilerContext;
use crate::error::SteelErr;
use log::{error, trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Imp<ID> = Arc<Mutex<dyn FnMut(&mut EvalState<ID>) -> Value<ID>>>;

#[derive(Clone)]
pub struct Impl<ID> {
    name: &'static str,
    imp: Imp<ID>,
}

impl<ID> Impl<ID> {
    fn new<F: 'static + FnMut(&mut EvalState<ID>) -> Value<ID>>(name: &'static str, imp: F) -> Self {
        Self {
            name,
            imp: Arc::new(Mutex::new(imp)),
        }
    }
}

impl<ID> std::fmt::Debug for Impl<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "<{}>", self.name)
    }
}

#[derive(Clone, Debug)]
pub enum Value<ID> {
    I64(i64), // a raw i64 value.
    // TODO: Func(ID), // reference to an 'expression' like thing that can be evaluated in some context.
    Extern(Impl<ID>), // reference to an extern...
}

impl<ID> Default for Value<ID> {
    fn default() -> Self {
        Self::I64(0)
    }
}

#[derive(Debug)]
pub struct EvalState<ID> {
    pub function_stack: Vec<(ID, usize, usize)>, // name -> memory address to store result.
    // Record all the bindings (i.e. name->index in memory stack).
    pub bindings: HashMap<String, Vec<usize>>, // name -> memory address to load result.
    pub mem_stack: Vec<Value<ID>>,                     // results.
}

impl<ID: std::fmt::Debug> EvalState<ID> {
    fn register_extern(mut self, imp: Impl<ID>) -> Self {
        // add binding
        let name = imp.name;
        let index = self.bind_mem(Value::Extern(imp));
        self.bind_name(name, index);
        self
    }
    fn run_extern(&mut self, name: &str) -> Value<ID> {
        // Get the Arc<Mutex<ImpFn>>
        let imp = self.get_value_for(name).unwrap_or_else(||panic!("couldn't find {}", name));
        if let Value::Extern(imp) = imp {
            let imp = imp.imp.clone();
            let mut imp = imp.lock().unwrap(); // Get the ImpFn.
            imp(self) // Run it
        } else {
            panic!("Expected an extern {:?}", imp)
        }
    }
}

impl<ID: std::fmt::Debug> Default for EvalState<ID> {
    fn default() -> Self {
        Self {
            function_stack: Vec::new(),
            bindings: HashMap::new(),
            mem_stack: Vec::new(),
        }
        .register_extern(Impl::new("putchar", |state: &mut EvalState<ID>| {
            let i = if let Some(Value::I64(i)) = state.get_value_for("arg_0") {
                i
            } else {
                return Value::I64(0);
            };
            if let Some(c) = char::from_u32(*i as u32) {
                print!("{}", c);
                Value::I64(1)
            } else {
                Value::I64(0)
            }
        }))
    }
}
impl<ID> EvalState<ID> {
    pub fn setup_call_to(&mut self, expr: ID, res: usize, args: usize) -> usize {
        self.function_stack.push((expr, res, args)); // to evaluate...
        res
    }

    pub fn bind_mem(&mut self, value: Value<ID>) -> usize {
        let index = self.mem_stack.len();
        self.mem_stack.push(value);
        index
    }

    pub fn bind_name(&mut self, name: &str, index: usize) {
        let entries = self.bindings.entry(name.to_string()).or_insert_with(Vec::new);
        entries.push(index); // Vec allows shadowing
    }

    pub fn setup_call(&mut self, expr: ID, args: usize) -> usize {
        let index = self.bind_mem(Value::default()); // assume it's a 0...
        self.setup_call_to(expr, index, args)
    }

    pub fn get_value_for(&mut self, name: &str) -> Option<&Value<ID>> {
        let mut bindings = self.bindings.get(name).cloned().unwrap_or_default();
        while let Some(binding) = bindings.last() {
            if *binding < self.mem_stack.len() {
                return Some(&self.mem_stack[*binding]);
            }
            bindings.pop();
        }
        None
    }
}

pub fn eval<C: CompilerContext>(context: &C, state: &mut EvalState<C::ID>) -> Result<(), C::E>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    while let Some((f, res_addr, args)) = state.function_stack.pop() {
        trace!("Step {:?}:", f);
        trace!("{}", context.pretty(f));
        step(context, state, f, res_addr, args)?;
    }
    Ok(())
}

pub fn step<C: CompilerContext>(
    context: &C,
    state: &mut EvalState<C::ID>,
    id: C::ID,
    res_index: usize,
    args: usize, // number of args to pop
) -> Result<(), C::E>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
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

fn bin_op<C: CompilerContext, F: FnOnce(i64, i64) -> i64>(
    _context: &C,
    state: &mut EvalState<C::ID>,
    name: &str,
    op: F,
) -> Value<C::ID> {
    let l = state.get_value_for("arg_0");
    let l = if let Some(Value::I64(l)) = l {
        *l
    } else {
        todo!("{} expects two i64 arguments got left: {:?}", name, &l);
    };
    let r = state.get_value_for("arg_1");
    let r = if let Some(Value::I64(r)) = r {
        *r
    } else {
        todo!("{} expects two i64 arguments got right: {:?}", name, &r);
    };
    Value::I64(op(l, r))
}

pub fn perform<C: CompilerContext>(
    context: &C,
    state: &mut EvalState<C::ID>,
    id: C::ID,
    res_index: usize,
    args: usize, // number of args to pop
) -> Result<Option<Value<C::ID>>, C::E>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    if let Ok(v) = context.get_i64(id) {
        return Ok(Some(Value::I64(*v)));
    }
    if let Ok(s) = context.get_symbol(id) {
        // if let Some(bound) = s.bound_to {
        // state.function_stack.push((bound, res_index));
        // Ok(())
        // }
        let r = if let Some(value) = state.get_value_for(&s.name) {
            value.clone()
        } else {
            match &*s.name {
                "putchar" => state.run_extern("putchar"),
                "+" => bin_op(context, state, "Addition", |l, r| l + r),
                "-" => bin_op(context, state, "Subtraction", |l, r| l - r),
                "*" => bin_op(context, state, "Multiplication", |l, r| l * r),
                "/" => bin_op(context, state, "Division", |l, r| l / r),
                _ => todo!("Unknown variable: {}", s.name),
            }
        };
        return Ok(Some(r));
    }
    if let Ok(c) = context.get_call(id) {
        // load in all the args
        let result = state.setup_call_to(c.callee, res_index, args + c.args.len());
        trace!("  inner {:?} -> {}", &result, context.pretty(c.callee));
        for (name, arg) in c.args.iter().rev() {
            trace!("    arg {:?} -> {}", &name, context.pretty(*arg));
            let index = state.setup_call(*arg, 0);
            state.bind_name(name, index);
        }
        return Ok(None);
    }
    // format!("{{node? {:?}}}", id)
    error!("Unknown node {}, {:?}", context.pretty(id), id);
    todo!("Unknown node {:?}", id);
}
