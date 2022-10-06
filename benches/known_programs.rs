use criterion::{black_box, criterion_group, criterion_main, Criterion};

use steel::{ast, ecs, handle, CompilerContext, SteelErr, Tasks};

fn criterion_benchmark_with<T: CompilerContext>(name: &'static str, c: &mut Criterion)
where
    SteelErr: From<<T as CompilerContext>::E>,
{
    c.bench_function(&format!("{} 123", name), |b| {
        b.iter(|| handle::<T>(black_box(Tasks::all("123"))).unwrap())
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    criterion_benchmark_with::<ast::Ast>("ast", c);
    criterion_benchmark_with::<ecs::Ecs>("ecs", c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
