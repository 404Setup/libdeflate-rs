use libdeflate::{Compressor, Decompressor};
use std::io;

#[test]
fn test_memory_limit() {
    let mut decompressor = Decompressor::new();
    // Simulate a large expected size for a small input
    let data = [0u8; 10];
    let expected_size = 1_000_000; // 1MB, which is > 10 * 2000 + 4096 = 24096

    // This should fail with the current logic because 1MB > limit (24KB)
    let result = decompressor.decompress_deflate(&data, expected_size);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("safety limit"));
}

#[test]
fn test_memory_limit_bypass_fixed() {
    let mut decompressor = Decompressor::new();

    // Set a strict memory limit of 50MB
    decompressor.set_max_memory_limit(50 * 1024 * 1024);

    // 1MB input -> 2GB output theoretically allowed by ratio check.
    let data = vec![0u8; 1024 * 1024]; // 1MB
    let expected_size = 100 * 1024 * 1024; // 100MB

    // The limit ratio check passes: 1MB * 2000 = 2GB > 100MB.
    // But the max memory limit (50MB) should catch it.

    let result = decompressor.decompress_deflate(&data, expected_size);

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("maximum memory limit"));
}

#[test]
fn test_valid_decompression_within_limit() {
    let mut decompressor = Decompressor::new();
    decompressor.set_max_memory_limit(1024 * 1024); // 1MB limit

    // Valid small data
    // Use the compressor to make valid data
    let mut compressor = Compressor::new(1).unwrap();
    let original = b"Hello world".repeat(10);
    let compressed = compressor.compress_deflate(&original).unwrap();

    let result = decompressor.decompress_deflate(&compressed, original.len());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), original);
}
