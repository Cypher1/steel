use criterion::{criterion_group, criterion_main, Criterion};
use steel::{ast, ecs, gen_code::Spec };
use log::trace;

mod benchmark_types;
use benchmark_types::*;

fn criterion_benchmark(c: &mut Criterion) {
    let _ = env_logger::builder().is_test(true).try_init();

    let mut programs: Vec<(usize, String, String)> = vec![
        (1, "known program constant".to_string(), "123".to_string()),
    ];

    let mut plus_chain = "1".to_string();
    let mut mul_chain = "1".to_string();
    let mut size = 1;
    let mut last = 1;
    for i in 0..500 {
        plus_chain = format!("{}+{}", i+2, plus_chain);
        mul_chain = format!("{}*{}", i+2, mul_chain);
        size += 3; // 1 = op, 1 = value i, 1 = the call.
        if size >= (10*last) {
            last = size;
            let spec = Spec::default().sized(size);
            let bench_type = format!("known program {}: {}", render_size(&spec), "plus chain");
            programs.push((size, bench_type, plus_chain.clone()));
            let bench_type = format!("known program {}: {}", render_size(&spec), "mul chain");
            programs.push((size, bench_type, mul_chain.clone()));
        }
    }

    for (size, bench_type, program) in programs {
        trace!("Generating benchmark of size {}. {}. Program:\n{}", size, bench_type, program);
        let spec = Spec::default().sized(size);
        benchmarks::<ast::Ast>("ast", &bench_type, &program, &spec, c);
        benchmarks::<ecs::Ecs>("ecs", &bench_type, &program, &spec, c);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
