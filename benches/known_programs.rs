use criterion::{criterion_group, criterion_main, Criterion};
use steel::{ast, ecs, gen_code::Spec };

mod benchmark_types;
use benchmark_types::*;

const PROGRAMS: &[(usize, &'static str)] = &[
    (1, "123"),
    (10, "1+2+3+4+5"),
    (10, "1*2*3*4*5"),
    (20, "1+2+3+4+5+6+7+8+9+10+11+12+13+14+15+16+17+18+19+20"),
    (20, "1*2*3*4*5*6*7*8*9*10*11*12*13*14*15*16*17*18*19*20"),
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
