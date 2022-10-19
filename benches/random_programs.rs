use criterion::{criterion_group, criterion_main, Criterion};
use steel::{
    ast, ecs,
    gen_code::{generate_random_program, Spec},
    CompilerContext,
};

mod benchmark_types;
use benchmark_types::*;

fn criterion_benchmark(c: &mut Criterion) {
    let _ = env_logger::builder().is_test(true).try_init();
    let mut rng = rand::thread_rng();
    for i in 0..4 {
        let size: usize = 10usize.pow(i);
        let spec = Spec::default().sized(size);
        let mut store = ast::Ast::new();
        let program = generate_random_program("ast generator", &mut store, &spec, &mut rng);
        let program = store.pretty(program);
        let bench_type = format!("random program {}", render_size(&spec));
        benchmarks::<ast::Ast>("ast", &bench_type, &program, &spec, c);
        benchmarks::<ecs::Ecs>("ecs", &bench_type, &program, &spec, c);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
