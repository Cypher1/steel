use crate::{
    nodes::{Call, Symbol},
    CompilerContext,
};
use rand::{rngs::ThreadRng, Rng};

pub struct Spec<ID> {
    size: usize,
    symbols: Vec<(String, ID, usize)>,
}

impl<ID> Default for Spec<ID> {
    fn default() -> Self {
        Self {
            size: 1,
            symbols: vec![],
        }
    }
}

impl<ID> Spec<ID> {
    pub fn add_symbol(mut self, name: String, bound_to: ID, arity: usize) -> Self {
        self.symbols.push((name, bound_to, arity));
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
    spec: Spec<Ctx::ID>,
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
                let arg_id = generate_random_program(_name, store, arg_spec, rng);
                let arg_name = rng
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(4)
                    .map(char::from)
                    .collect();
                inner_spec = inner_spec.add_symbol(arg_name, arg_id, 0);
                args.push(arg_id);
            }
        }
        let callee = generate_random_program(_name, store, inner_spec, rng);
        return store.add(Call { callee, args });
    }
    let symbol_index: usize = rng.gen_range(0..=spec.symbols.len());
    if symbol_index < spec.symbols.len() {
        let (name, bound_to, _arity) = &spec.symbols[symbol_index];
        return store.add(Symbol {
            name: name.to_string(),
            is_operator: rng.gen(),
            bound_to: Some(*bound_to),
        });
    }
    let value: i64 = rng.gen();
    store.add(value)
}
