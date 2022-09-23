use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use std::sync::Mutex;
use steel::{
    ast, ecs, gen_code::{generate_random_program, Spec}, handle, CompilerContext, SteelErr,
};

// Work around for static lifetime data
lazy_static! {
    static ref PROGRAMS: Mutex<Vec<String>> = {
        let mut v = vec![];
        v.reserve(1000); // Ensure that we don't invalidate pointers into the vector.
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

// End workaround for static lifetime data

fn criterion_benchmark_with<'source, T: CompilerContext<'source>>(
    name: &'static str,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext<'source>>::E>,
{
    let mut rng = rand::thread_rng();
    let mut store = T::new();
    for i in 0..4 {
        let size: usize = 10usize.pow(i);
        let spec = Spec::default().sized(size);
        let program = generate_random_program(name, &mut store, spec, &mut rng);
        // This ensures that the program is stored for a static lifetime, because criterion
        // benchmark functions live longer than the lifetime of this function, which means we need
        // a home for the test data.
        let program = get_program(save_program(store.pretty(program)));

        eprintln!("testing {} with {}\n{}", name, size, program);
        c.bench_function(&format!("{} parse random program {}", name, size), |b| {
            b.iter(|| handle::<T>(black_box(program)))
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    criterion_benchmark_with::<ast::Ast>("ast", c);
    criterion_benchmark_with::<ecs::Ecs>("ecs", c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
