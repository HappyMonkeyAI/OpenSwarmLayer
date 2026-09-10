use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::Write;
use tempfile::NamedTempFile;
use ts_core::sha256;
use ts_format::plan_chunks;

fn benchmark_chunk_planning(c: &mut Criterion) {
    c.bench_function("plan_chunks_64MiB", |b| {
        b.iter(|| black_box(plan_chunks(64 * 1024 * 1024, 4096, 16 * 1024 * 1024).unwrap()))
    });
}

fn benchmark_hashing(c: &mut Criterion) {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(&vec![7_u8; 4 * 1024 * 1024]).unwrap();
    let bytes = std::fs::read(file.path()).unwrap();
    c.bench_function("sha256_4MiB", |b| b.iter(|| black_box(sha256(&bytes))));
}

criterion_group!(benches, benchmark_chunk_planning, benchmark_hashing);
criterion_main!(benches);
