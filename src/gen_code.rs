use crate::{
    nodes::{Call, Symbol},
    CompilerContext,
};
use std::collections::HashMap;
use rand::{distributions::Alphanumeric, rngs::ThreadRng, Rng};

pub struct Spec {
    pub size: usize,
    pub arity: usize,
    symbols: HashMap<usize, Vec<(String, bool)>>,
}

static CHANCE_OF_POTENTIALLY_LARGE_CONSTANT: f64 = 0.05;
static CHANCE_OF_SYMBOL: f64 = 0.25;

fn weighted_bool(rng: &mut ThreadRng, chance: f64) -> bool {
    rng.gen_range(0f64..=1f64) < chance
}

impl Default for Spec {
    fn default() -> Self {
        let bin_ops = vec![
            ("+".to_string(), true),
            ("*".to_string(), true),
            ("/".to_string(), true),
            ("-".to_string(), true),
        ];
        let mut symbols = HashMap::new();
        symbols.insert(2, bin_ops);
        Self {
            size: 1,
            arity: 0,
            symbols,
        }
    }
}

impl Spec {
    pub fn add_symbol(mut self, name: String, is_operator: bool, arity: usize) -> Self {
        let symbols = self.symbols.entry(arity).or_insert_with(Vec::new);
        symbols.push((name, is_operator));
        self
    }
    pub fn arity(mut self, arity: usize) -> Self {
        self.arity = arity;
        self
    }
    pub fn sized(mut self, size: usize) -> Self {
        self.size = size;
        self
    }
}

pub fn generate_random_program<Ctx: CompilerContext>(
    _name: &'static str,
    store: &mut Ctx,
    spec: &Spec,
    rng: &mut ThreadRng,
) -> Ctx::ID {
    let r = generate_random_program_impl::<Ctx>(_name, store, spec, rng);
    // eprintln!(">> {}", store.pretty(r));
    r
}

pub fn generate_random_program_impl<Ctx: CompilerContext>(
    _name: &'static str,
    store: &mut Ctx,
    spec: &Spec,
    rng: &mut ThreadRng,
) -> Ctx::ID {
    if spec.size > 1 {
        let mut args = vec![];
        let mut args_size: usize = rng.gen_range(1..spec.size);
        let inner_size: usize = spec.size - args_size - 1;
        let mut inner_spec = Spec::default().sized(inner_size);
        if args_size > 0 {
            let num_args: usize = rng.gen_range(1..=args_size);
            args_size -= num_args; // at least one node per arg.
            let mut arg_index = 0;
            for _ in 0..num_args {
                let arg_size: usize = rng.gen_range(1..=1 + args_size);
                let arg_spec = Spec::default().sized(arg_size);
                args_size -= arg_size - 1;
                let arg_id = generate_random_program(_name, store, &arg_spec, rng);
                let tail: String = rng
                    .sample_iter(&Alphanumeric)
                    .take(3)
                    .map(char::from)
                    .collect();
                let arg_name = if rng.gen() {
                    rng.gen_range('a'..='z').to_string() + &tail
                } else {
                    let s = format!("arg_{}", arg_index);
                    arg_index+=1;
                    s
                };
                inner_spec = inner_spec.add_symbol(arg_name.clone(), false, 0);
                args.push((arg_name, arg_id));
            }
        }
        inner_spec = inner_spec.arity(args.len());
        let callee = generate_random_program(_name, store, &inner_spec, rng);
        return store.add(Call { callee, args });
    }
    if !spec.symbols.is_empty() && weighted_bool(rng, CHANCE_OF_SYMBOL) {
        if let Some(symbols) = &spec.symbols.get(&spec.arity) {
            let symbol_index: usize = rng.gen_range(0..symbols.len());
            let (name, is_operator) = &symbols[symbol_index];
            let bound_to = None; // TODO:
            return store.add(Symbol {
                name: name.to_string(),
                is_operator: *is_operator,
                bound_to,
            });
        }
    }
    let value: i64 = if weighted_bool(rng, CHANCE_OF_POTENTIALLY_LARGE_CONSTANT) {
        rng.gen() // some potentially large constant.
    } else {
        rng.gen_range(-5i64..=5i64) // some small value.
    };
    store.add(value)
}
