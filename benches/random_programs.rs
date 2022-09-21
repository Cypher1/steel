use criterion::{black_box, criterion_group, criterion_main, Criterion};

use steel::{ast, ecs, handle};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ast 123", |b| b.iter(|| handle::<ast::Ast>(black_box("123")).unwrap()));
    c.bench_function("ecs 123", |b| b.iter(|| handle::<ecs::Ecs>(black_box("123")).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
