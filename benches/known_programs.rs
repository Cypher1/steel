use criterion::{criterion_group, criterion_main, Criterion};
use steel::{ast, ecs, gen_code::Spec };

mod benchmark_types;
use benchmark_types::*;

const PROGRAMS: &[(usize, &'static str)] = &[
    (1, "123"),
];

fn criterion_benchmark(c: &mut Criterion) {
    let _ = env_logger::builder().is_test(true).try_init();

    for (size, program) in PROGRAMS {
        let spec = Spec::default().sized(*size);
        let bench_type = format!("known program {}: {}", render_size(&spec), program);
        benchmarks::<ast::Ast>("ast", &bench_type, &program, &spec, c);
        benchmarks::<ecs::Ecs>("ecs", &bench_type, &program, &spec, c);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
