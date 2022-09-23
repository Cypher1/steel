use criterion::{black_box, criterion_group, criterion_main, Criterion};
use steel::{
    ast, ecs,
    gen_code::{generate_random_program, Spec},
    handle, CompilerContext, SteelErr,
};

fn criterion_benchmark_with<T: CompilerContext>(
    name: &'static str,
    program: &str,
    spec: &Spec<T::ID>,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    eprintln!("testing {} with {}\n{}", name, spec.size, program);
    c.bench_function(
        &format!("{} parse random program {}", name, spec.size),
        |b| b.iter(|| handle::<T>(black_box(&program))),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    for i in 0..3 {
        let size: usize = 10usize.pow(i);
        let spec = Spec::default().sized(size);
        let mut store = ast::Ast::new();
        let program = generate_random_program("ast generator", &mut store, &spec, &mut rng);
        let program = store.pretty(program);
        criterion_benchmark_with::<ast::Ast>("ast", &program, &spec, c);
        criterion_benchmark_with::<ecs::Ecs>("ecs", &program, &spec, c);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
