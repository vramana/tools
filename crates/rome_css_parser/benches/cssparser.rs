use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rome_css_parser::parse;
use std::fs::read_to_string;
use std::path::PathBuf;
pub fn criterion_benchmark(c: &mut Criterion) {
    let src_dir: PathBuf = ["benches", "test.css"].iter().collect();
    let source = read_to_string(src_dir).unwrap();
    c.bench_function("cssparser bench", |b| {
        b.iter(|| parse(black_box(source.as_str())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
