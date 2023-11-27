#![allow(unused)]
fn main() {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    pub fn criterion_benchmark(c: &mut Criterion) {
        c.bench_function("fib 20", |b| b.iter(|| assert!(true)));
    }

    criterion_group!(benches, criterion_benchmark);
    criterion_main!(benches);
}
