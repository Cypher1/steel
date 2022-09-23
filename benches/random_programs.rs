use criterion::{black_box, criterion_group, criterion_main, Criterion};
use steel::{
    ast, ecs,
    gen_code::{generate_random_program, Spec},
    handle, CompilerContext, SteelErr,
};

fn criterion_benchmark_with<T: CompilerContext>(name: &'static str, c: &mut Criterion)
where
    SteelErr: From<<T as CompilerContext>::E>,
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
        let program = store.pretty(program);

        eprintln!("testing {} with {}\n{}", name, size, program);
        c.bench_function(&format!("{} parse random program {}", name, size), |b| {
            b.iter(|| handle::<T>(black_box(&program)))
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    criterion_benchmark_with::<ast::Ast>("ast", c);
    criterion_benchmark_with::<ecs::Ecs>("ecs", c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
