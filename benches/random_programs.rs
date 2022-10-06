use criterion::{black_box, criterion_group, criterion_main, Criterion};
use steel::{
    ast, ecs,
    gen_code::{generate_random_program, Spec},
    handle, handle_steps, CompilerContext, SteelErr, Tasks
};

fn benchmark_parse<T: CompilerContext>(
    name: &'static str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    eprintln!("testing {} with {}\n{}", name, spec.size, program);
    c.bench_function(
        &format!("{} parse random program {}", name, spec.size),
        |b| b.iter(|| handle::<T>(black_box(Tasks::parse(program))))
    );
}

fn benchmark_eval<T: CompilerContext>(
    name: &'static str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    eprintln!("testing {} with {}\n{}", name, spec.size, program);
    c.bench_function(
        &format!("{} eval random program {}", name, spec.size),
        |b| {
            let mut store = T::new();
            let (id, _res) = handle_steps::<T>(&mut store, Tasks::parse(program)).expect("Should parse program without error");
            let id = id.expect("Should have parsed a program");
            b.iter(|| handle_steps::<T>(&mut store, black_box(Tasks::pre_parsed(id).and_eval())))
        }
    );
}

fn benchmark_parse_and_eval_tasks<T: CompilerContext>(
    name: &'static str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    eprintln!("testing {} with {}\n{}", name, spec.size, program);
    c.bench_function(
        &format!("{} parse and eval random program {}", name, spec.size),
        |b| b.iter(|| handle::<T>(black_box(Tasks::parse(program).and_eval())))
    );
}

fn benchmarks<T: CompilerContext>(name: &'static str,
    program: &str,
    spec: &Spec,
    c: &mut Criterion,
) where
    SteelErr: From<<T as CompilerContext>::E>,
{
    benchmark_parse::<T>(name, &program, &spec, c);
    benchmark_eval::<T>(name, &program, &spec, c);
    benchmark_parse_and_eval_tasks::<T>(name, &program, &spec, c);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    for i in 0..4 {
        let size: usize = 10usize.pow(i);
        let spec = Spec::default().sized(size);
        let mut store = ast::Ast::new();
        let program = generate_random_program("ast generator", &mut store, &spec, &mut rng);
        let program = store.pretty(program);

        benchmarks::<ast::Ast>("ast", &program, &spec, c);
        benchmarks::<ecs::Ecs>("ecs", &program, &spec, c);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
