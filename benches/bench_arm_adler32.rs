use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(target_arch = "aarch64")]
use libdeflate::adler32::adler32;

#[cfg(target_arch = "aarch64")]
fn bench_adler32(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024]; // 1MB
    c.bench_function("adler32_1mb", |b| {
        b.iter(|| adler32(black_box(1), black_box(&data)))
    });
}

#[cfg(not(target_arch = "aarch64"))]
fn bench_adler32(_: &mut Criterion) {}

criterion_group!(benches, bench_adler32);
criterion_main!(benches);
