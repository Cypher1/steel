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

    let mut plus_tree = "1".to_string();
    // let mut mul_tree = "1".to_string();
    let mut size = 1;
    let mut last = 1;
    while size < 1000000 {
        plus_tree = format!("({})+({})", plus_tree, plus_tree);
        // mul_tree = format!("({})*({})", mul_tree, mul_tree);
        size = size*2+2; // 2*size = args, 1= the op, 1 = the call.
        if size > 1000 && size >= (10*last) {
            last = size;
            let spec = Spec::default().sized(size);
            let bench_type = format!("known program {}: {}", render_size(&spec), "plus tree");
            programs.push((size, bench_type, plus_tree.clone()));
            // let bench_type = format!("known program {}: {}", render_size(&spec), "mul tree");
            // programs.push((size, bench_type, mul_tree.clone()));
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
