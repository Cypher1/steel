use criterion::{black_box, Criterion};
use log::debug;
use steel::{gen_code::Spec, handle, handle_steps, CompilerContext, SteelErr, Tasks};

pub fn render_size(spec: &Spec) -> String {
    spec.size
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string())
}

pub fn benchmark_parse<T: CompilerContext>(
    name: &'static str,
    bench_type: &str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    c.bench_function(&format!("{} parse {}", name, bench_type), |b| {
        debug!("testing {} with {}\n{}", name, render_size(spec), program);
        b.iter(|| handle::<T>(black_box(Tasks::parse(program))))
    });
}

pub fn benchmark_optimize<T: CompilerContext>(
    name: &'static str,
    bench_type: &str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    c.bench_function(&format!("{} optimize {}", name, bench_type), |b| {
        debug!("testing {} with {}\n{}", name, render_size(spec), program);
        let mut store = T::new();
        let (id, _res) = handle_steps::<T>(&mut store, Tasks::parse(program))
            .expect("Should parse program without error");
        let id = id.expect("Should have parsed a program");
        b.iter(|| handle_steps::<T>(&mut store, black_box(Tasks::pre_parsed(id).and_optimize())))
    });
}

pub fn benchmark_eval_pre_optimized<T: CompilerContext>(
    name: &'static str,
    bench_type: &str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    c.bench_function(
        &format!("{} eval pre-optimized {}", name, bench_type),
        |b| {
            debug!("testing {} with {}\n{}", name, render_size(spec), program);
            let mut store = T::new();
            let (id, _res) = handle_steps::<T>(&mut store, Tasks::parse(program).and_optimize())
                .expect("Should parse program without error");
            let id = id.expect("Should have parsed a program");
            b.iter(|| handle_steps::<T>(&mut store, black_box(Tasks::pre_parsed(id).and_eval())))
        },
    );
}

pub fn benchmark_eval<T: CompilerContext>(
    name: &'static str,
    bench_type: &str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    c.bench_function(&format!("{} eval {}", name, bench_type), |b| {
        debug!("testing {} with {}\n{}", name, render_size(spec), program);
        let mut store = T::new();
        let (id, _res) = handle_steps::<T>(&mut store, Tasks::parse(program))
            .expect("Should parse program without error");
        let id = id.expect("Should have parsed a program");
        b.iter(|| handle_steps::<T>(&mut store, black_box(Tasks::pre_parsed(id).and_eval())))
    });
}

pub fn benchmark_parse_and_eval_tasks<T: CompilerContext>(
    name: &'static str,
    bench_type: &str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    c.bench_function(&format!("{} parse and eval {}", name, bench_type), |b| {
        debug!("testing {} with {}\n{}", name, render_size(spec), program);
        b.iter(|| handle::<T>(black_box(Tasks::parse(program).and_eval())))
    });
}

pub fn benchmarks<T: CompilerContext>(
    name: &'static str,
    bench_type: &str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    //benchmark_parse::<T>(name, bench_type, program, spec, c);
    benchmark_optimize::<T>(name, bench_type, program, spec, c);
    //benchmark_eval::<T>(name, bench_type, program, spec, c);
    //benchmark_eval_pre_optimized::<T>(name, bench_type, program, spec, c);
    //benchmark_parse_and_eval_tasks::<T>(name, bench_type, program, spec, c);
}
