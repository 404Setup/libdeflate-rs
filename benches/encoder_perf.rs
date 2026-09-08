use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use libdeflate::stream::DeflateEncoder;
use std::io::Write;

fn bench_encoder_parallel(c: &mut Criterion) {
    let size = 10 * 1024 * 1024; // 10MB
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        data.push((i % 256) as u8);
    }

    let mut group = c.benchmark_group("DeflateEncoder Parallel");
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("write_all 10MB", |b| {
        b.iter(|| {
            let sink = std::io::sink();
            let mut encoder = DeflateEncoder::new(sink, 6); // Default 1MB buffer
            encoder.write_all(&data).unwrap();
            encoder.finish().unwrap();
        });
    });

    group.finish();
}

fn bench_stored_blocks(c: &mut Criterion) {
    let data = vec![42; 1024 * 1024];
    let mut compressor = libdeflate::Compressor::new(0).unwrap();
    let mut output = vec![0; compressor.deflate_compress_bound(data.len())];
    c.bench_function("stored_blocks_1mb", |b| {
        b.iter(|| {
            compressor
                .compress_deflate_into(std::hint::black_box(&data), &mut output)
                .unwrap()
        });
    });
    c.bench_function("stored_encoder_1mb", |b| {
        b.iter(|| {
            let mut encoder = DeflateEncoder::new(std::io::sink(), 0);
            encoder.write_all(std::hint::black_box(&data)).unwrap();
            encoder.finish().unwrap();
        });
    });
}

criterion_group!(benches, bench_encoder_parallel, bench_stored_blocks);
criterion_main!(benches);
