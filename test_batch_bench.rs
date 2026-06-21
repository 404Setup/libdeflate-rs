use libdeflate::batch::{BatchCompressor, BatchDecompressor};
use std::time::Instant;
use std::hint::black_box;

fn main() {
    let mut inputs = Vec::new();
    let data1 = b"Hello world. This is the first string.".repeat(100);
    for _ in 0..100000 {
        inputs.push(data1.as_slice());
    }

    let compressor = BatchCompressor::new(6);
    let compressed_data = compressor.compress_batch(&inputs);

    let max_sizes: Vec<usize> = inputs.iter().map(|i| i.len()).collect();
    let compressed_refs: Vec<&[u8]> = compressed_data.iter().map(|v| v.as_slice()).collect();

    let decompressor = BatchDecompressor::new();

    let start = Instant::now();
    for _ in 0..10 {
        let decompressed_results = decompressor.decompress_batch(&compressed_refs, &max_sizes);
        black_box(decompressed_results);
    }
    let duration = start.elapsed();
    println!("Decompression took: {:?}", duration);
}
