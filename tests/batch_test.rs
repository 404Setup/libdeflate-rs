use libdeflate::batch::{BatchCompressor, BatchDecompressor};

#[test]
fn test_batch_allocation_failure_preserves_other_results() {
    let compressed = BatchCompressor::new(1).compress_batch(&[b"hello"]);
    let input = compressed[0].as_slice();
    assert_eq!(
        BatchDecompressor::new().decompress_batch(&[input, input, input], &[5, usize::MAX, 5]),
        vec![Some(b"hello".to_vec()), None, Some(b"hello".to_vec())]
    );
}

#[test]
fn test_batch_missing_sizes_preserves_input_positions() {
    let compressed = BatchCompressor::new(1).compress_batch(&[b"hello", b"world"]);
    let inputs: Vec<&[u8]> = compressed.iter().map(Vec::as_slice).collect();
    let decoder = BatchDecompressor::new();
    assert_eq!(
        decoder.decompress_batch(&inputs, &[5]),
        vec![Some(b"hello".to_vec()), None]
    );
    assert_eq!(decoder.decompress_batch(&inputs, &[]), vec![None, None]);
    assert_eq!(
        decoder.decompress_batch(&inputs[..1], &[5, 5]),
        vec![Some(b"hello".to_vec())]
    );
}

#[test]
fn test_batch_compress_decompress_roundtrip() {
    let inputs: Vec<&[u8]> = vec![
        b"Hello world! This is a test string for deflate compression.",
        b"Another test string.",
        b"Repeating pattern repeating pattern repeating pattern repeating pattern.",
        b"Short",
        &[0u8; 1000],
    ];

    let compressor = BatchCompressor::new(6);
    let compressed_batch = compressor.compress_batch(&inputs);

    assert_eq!(compressed_batch.len(), inputs.len());

    let max_out_sizes: Vec<usize> = inputs.iter().map(|input| input.len()).collect();
    let compressed_refs: Vec<&[u8]> = compressed_batch.iter().map(|v| v.as_slice()).collect();

    let decompressor = BatchDecompressor::new();
    let decompressed_batch = decompressor.decompress_batch(&compressed_refs, &max_out_sizes);

    assert_eq!(decompressed_batch.len(), inputs.len());

    for (i, result) in decompressed_batch.iter().enumerate() {
        match result {
            Some(decompressed) => {
                assert_eq!(
                    decompressed.as_slice(),
                    inputs[i],
                    "Mismatch at index {}",
                    i
                );
            }
            None => panic!("Decompression failed for input index {}", i),
        }
    }
}

#[test]
fn test_batch_empty() {
    let compressor = BatchCompressor::new(6);
    let compressed = compressor.compress_batch(&[]);
    assert!(compressed.is_empty());

    let decompressor = BatchDecompressor::new();
    let decompressed = decompressor.decompress_batch(&[], &[]);
    assert!(decompressed.is_empty());
}

#[test]
fn test_batch_empty_input() {
    let inputs: Vec<&[u8]> = vec![b"", b"Not empty"];
    let compressor = BatchCompressor::new(6);
    let compressed = compressor.compress_batch(&inputs);

    assert_eq!(compressed.len(), 2);
    assert!(!compressed[0].is_empty());

    let max_out_sizes = vec![0, 9];
    let compressed_refs: Vec<&[u8]> = compressed.iter().map(|v| v.as_slice()).collect();

    let decompressor = BatchDecompressor::new();
    let decompressed = decompressor.decompress_batch(&compressed_refs, &max_out_sizes);

    assert_eq!(decompressed.len(), 2);
    assert_eq!(decompressed[0], Some(Vec::new()));
    assert_eq!(decompressed[1], Some(b"Not empty".to_vec()));
}

#[test]
fn test_batch_decompress_error() {
    let invalid_data = vec![0u8, 1, 2, 3, 4, 5];
    let inputs: Vec<&[u8]> = vec![&invalid_data];
    let max_out_sizes = vec![100];

    let decompressor = BatchDecompressor::new();
    let decompressed = decompressor.decompress_batch(&inputs, &max_out_sizes);

    assert_eq!(decompressed.len(), 1);
    assert_eq!(decompressed[0], None);
}

#[test]
fn test_batch_decompress_insufficient_buffer() {
    let input = b"Hello world!";
    let compressor = BatchCompressor::new(6);
    let compressed = compressor.compress_batch(&[input]);

    let compressed_refs: Vec<&[u8]> = compressed.iter().map(|v| v.as_slice()).collect();

    let max_out_sizes = vec![input.len() - 1];

    let decompressor = BatchDecompressor::new();
    let decompressed = decompressor.decompress_batch(&compressed_refs, &max_out_sizes);

    assert_eq!(decompressed.len(), 1);
    assert_eq!(decompressed[0], None);
}
