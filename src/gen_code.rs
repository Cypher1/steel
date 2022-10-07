use crate::{
    nodes::{Call, Symbol},
    CompilerContext,
};
use rand::{distributions::Alphanumeric, rngs::ThreadRng, Rng};

static CHANCE_OF_POTENTIALLY_LARGE_CONSTANT: f64 = 0.05;
static CHANCE_OF_SYMBOL: f64 = 0.5;
static CHANCE_OF_NAMED_ARG: f64 = 0.25;

fn weighted_bool(rng: &mut ThreadRng, chance: f64) -> bool {
    rng.gen_range(0f64..=1f64) < chance
}

#[derive(Clone)]
pub struct Spec {
    size: Option<usize>,
    name: String,
    is_operator: bool,
    in_scope: Vec<Spec>,
}

impl Spec {
    pub fn new(name: &str, is_operator: bool, in_scope: Vec<Spec>) -> Self {
        Self {
            name: name.to_string(),
            size: None,
            is_operator,
            in_scope,
        }
    }

    pub fn symbol(name: &str) -> Self {
        Self::new(name, false, Vec::new())
    }

    pub fn operator(name: &str, n_args: usize) -> Self {
        let mut in_scope = vec![];
        for i in 0..n_args {
            in_scope.push(Spec::symbol(&format!("arg_{}", i)));
        }
        Self::new(name, true, in_scope)
    }

    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn add_symbol(mut self, spec: Spec) -> Self {
        self.in_scope.push(spec);
        self
    }

    pub fn sized(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    fn is_in_scope(&self, _context: &Spec) -> bool {
        // TODO: check if all the required args are in `_context`.
        false
    }
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            name: "main".to_string(),
            is_operator: false,
            size: Some(1),
            in_scope: vec![
                Spec::symbol("putchar"),
                Spec::operator("+", 2),
                Spec::operator("*", 2),
                Spec::operator("/", 2),
                Spec::operator("-", 2),
            ],
        }
    }
}


pub fn generate_random_program<Ctx: CompilerContext>(
    _name: &'static str,
    store: &mut Ctx,
    spec: &Spec,
    rng: &mut ThreadRng,
) -> Ctx::ID {
    // eprintln!(">> {}", store.pretty(r));
    generate_random_program_impl::<Ctx>(_name, store, spec, rng)
}

pub fn generate_random_program_impl<Ctx: CompilerContext>(
    _name: &'static str,
    store: &mut Ctx,
    spec: &Spec,
    rng: &mut ThreadRng,
) -> Ctx::ID {
    let size = spec.size.unwrap_or_else(||rng.gen_range(1..1000));
    if size > 1 {
        let mut args = vec![];
        let mut args_size: usize = rng.gen_range(1..size);
        let inner_size: usize = size - args_size - 1;
        let mut inner_spec = spec.clone().named("self".to_string()).sized(inner_size);
        let arg_range = (args_size as f64).sqrt() as usize;
        let num_args: usize = rng.gen_range(0..=arg_range);
        args_size -= num_args; // at least one node per arg.
        let mut arg_index = 0;
        for _ in 0..num_args {
            let arg_size: usize = rng.gen_range(1..=1 + args_size);
            let arg_spec = spec.clone().sized(arg_size);
            args_size -= arg_size - 1;
            let arg_id = generate_random_program(_name, store, &arg_spec, rng);
            let tail: String = rng
                .sample_iter(&Alphanumeric)
                .take(3)
                .map(char::from)
                .collect();
            let arg_name = if weighted_bool(rng, CHANCE_OF_NAMED_ARG) {
                rng.gen_range('a'..='z').to_string() + &tail
            } else {
                let s = format!("arg_{}", arg_index);
                arg_index += 1;
                s
            };
            // assume no higher-order arguments.
            inner_spec = inner_spec.add_symbol(arg_spec);
            args.push((arg_name, arg_id));
        }
        let callee = generate_random_program(_name, store, &inner_spec, rng);
        return store.add(Call { callee, args });
    }
    let symbols: Vec<Spec> = spec.in_scope.iter().filter(|s| s.is_in_scope(spec)).cloned().collect();
    if !symbols.is_empty() && weighted_bool(rng, CHANCE_OF_SYMBOL) {
        let symbol_index: usize = rng.gen_range(0..symbols.len());
        let spec = &symbols[symbol_index];
        return store.add(Symbol {
            name: spec.name.to_string(),
            is_operator: spec.is_operator,
        });
    }
    let value: i64 = if weighted_bool(rng, CHANCE_OF_POTENTIALLY_LARGE_CONSTANT) {
        rng.gen() // some potentially large constant.
    } else {
        rng.gen_range(-5i64..=5i64) // some small value.
    };
    store.add(value)
}
