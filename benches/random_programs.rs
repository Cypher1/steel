use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng, rngs::ThreadRng};
use steel::{ast, ecs, handle, nodes::Call, CompilerContext, SteelErr};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref PROGRAMS: Mutex<Vec<String>> = {
        let mut v = vec![];
        v.reserve(1000);
        Mutex::new(v)
    };
}

fn save_program(program: String) -> usize {
    let mut guard = PROGRAMS.lock().unwrap();
    let id = guard.len();
    guard.push(program);
    id
}

fn get_program(id: usize) -> &'static String {
    let guard = PROGRAMS.lock().unwrap();
    unsafe {
        let ptr: *const String = &guard[id];
        &*ptr
    }
}

fn generate_random_program<'b, T: CompilerContext<'b>>(
    _name: &'static str,
    store: &mut T,
    size: usize,
    rng: &mut ThreadRng,
) -> T::ID {
    if size > 2 {
        let mut args_size: usize = rng.gen_range(0..size-2);
        let num_args: usize = rng.gen_range(0..=args_size);
        let mut args = vec![];
        for _ in 0..num_args {
            let arg_size: usize = rng.gen_range(0..=args_size);
            args_size -= arg_size;
            if args_size > 0 {
                args.push(generate_random_program(_name, store, args_size, rng));
            }
        }
        let num_inner: usize = size - num_args - 1;
        let callee = generate_random_program(_name, store, num_inner, rng);
        return store.add(Call {callee, args});
    }
    let value: i64 = rng.gen();
    store.add(value)
}

fn criterion_benchmark_with<'source, T: CompilerContext<'source>>(name: &'static str, c: &mut Criterion)
where
    SteelErr: From<<T as CompilerContext<'source>>::E>,
{
    let mut rng = rand::thread_rng();
    let mut store = T::new();

    for i in 0..4 {
        let size: usize = 10usize.pow(i);
        let program = generate_random_program::<T>(name, &mut store, size, &mut rng);
        let program = get_program(save_program(store.pretty(program)));

        eprintln!("testing {} with {}\n{}", name, size, program);
        c.bench_function(&format!("{} parse random program {}", name, size), |b| {
            b.iter(|| {
                    handle::<T>(black_box(program))
            })
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    criterion_benchmark_with::<ast::Ast>("ast", c);
    criterion_benchmark_with::<ecs::Ecs>("ecs", c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
