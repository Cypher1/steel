use crate::{
    nodes::{Call, Symbol},
    CompilerContext,
};

use rand::{distributions::Alphanumeric, rngs::ThreadRng, Rng};

pub struct Spec {
    pub size: usize,
    symbols: Vec<(String, bool, usize)>,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            size: 1,
            symbols: vec![
                ("+".to_string(), true, 2),
                ("*".to_string(), true, 2),
                ("/".to_string(), true, 2),
                ("-".to_string(), true, 2),
            ],
        }
    }
}

impl Spec {
    pub fn add_symbol(mut self, name: String, is_operator: bool, arity: usize) -> Self {
        self.symbols.push((name, is_operator, arity));
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
            args_size -= num_args; // at lesst one node per arg.
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
                let arg_name = rng.gen_range('a'..='z').to_string() + &tail;
                inner_spec = inner_spec.add_symbol(arg_name.clone(), false, 0);
                args.push((arg_name, arg_id));
            }
        }
        let callee = generate_random_program(_name, store, &inner_spec, rng);
        return store.add(Call { callee, args });
    }
    let chance_of_value: f64 = 0.25;
    if !spec.symbols.is_empty() && rng.gen_range(0f64..=1f64) >= chance_of_value {
        let symbol_index: usize = rng.gen_range(0..spec.symbols.len());
        let (name, is_operator, _arity) = &spec.symbols[symbol_index];
        let bound_to = None; // TODO:
        return store.add(Symbol {
            name: name.to_string(),
            is_operator: *is_operator,
            bound_to,
        });
    }
    let value: i64 = rng.gen();
    store.add(value)
}
