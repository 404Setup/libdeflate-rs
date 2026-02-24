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

#[test]
fn test_decompression_ratio_limit() {
    let mut decompressor = Decompressor::new();

    // Default limit is 2000:1 + 4096.
    // Let's create a small input.
    let input = [0u8; 10];
    // Limit = 10 * 2000 + 4096 = 24096.

    // Case 1: Within default limit (20000 <= 24096)
    // Should NOT return InvalidInput (might return InvalidData because input is garbage)
    let res = decompressor.decompress_deflate(&input, 20000);
    if let Err(e) = &res {
        assert_ne!(
            e.kind(),
            io::ErrorKind::InvalidInput,
            "Should not reject 20000 bytes for 10 bytes input with default ratio"
        );
    }

    // Case 2: Exceed default limit (30000 > 24096)
    // Should return InvalidInput
    let res = decompressor.decompress_deflate(&input, 30000);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().kind(),
        io::ErrorKind::InvalidInput,
        "Should reject 30000 bytes for 10 bytes input with default ratio"
    );

    // Case 3: Set custom limit ratio to 10
    decompressor.set_limit_ratio(10);
    // New limit = 10 * 10 + 4096 = 4196.

    // Case 4: Exceed custom limit (5000 > 4196)
    let res = decompressor.decompress_deflate(&input, 5000);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().kind(),
        io::ErrorKind::InvalidInput,
        "Should reject 5000 bytes for 10 bytes input with ratio 10"
    );

    // Case 5: Within custom limit (4000 <= 4196)
    let res = decompressor.decompress_deflate(&input, 4000);
    if let Err(e) = &res {
        assert_ne!(
            e.kind(),
            io::ErrorKind::InvalidInput,
            "Should not reject 4000 bytes for 10 bytes input with ratio 10"
        );
    }
}

#[test]
fn test_memory_limit_with_real_data() {
    let mut compressor = Compressor::new(1).unwrap();
    let mut decompressor = Decompressor::new();

    // Create a 1MB buffer of zeros (highly compressible)
    let original = vec![0u8; 1_000_000];
    let compressed = compressor.compress_deflate(&original).unwrap();
    // Compressed size should be very small (e.g. < 200 bytes)

    // Set max memory limit to 500KB (less than 1MB)
    decompressor.set_max_memory_limit(500_000);

    // Try to decompress, requesting 1MB
    let result = decompressor.decompress_deflate(&compressed, original.len());

    // Expect failure due to memory limit
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("maximum memory limit"));
}

#[test]
fn test_ratio_limit_with_real_data() {
    let mut decompressor = Decompressor::new();

    // Create a 1MB buffer of zeros
    let original = vec![0u8; 1_000_000];

    // Use level 12 for high compression
    let mut compressor = Compressor::new(12).unwrap();
    let compressed = compressor.compress_deflate(&original).unwrap();

    // Set ratio limit to a value that definitely fails.
    // With level 12, 1MB zeros compresses to ~1000 bytes.
    // Limit = 1000 * 100 + 4096 = 104096.
    // 1,000,000 > 104096.
    decompressor.set_limit_ratio(100);

    // Try to decompress high compression
    let result = decompressor.decompress_deflate(&compressed, original.len());

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("safety limit"));

    // Set ratio limit back to something permissive
    decompressor.set_limit_ratio(100_000);

    let result = decompressor.decompress_deflate(&compressed, original.len());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), original);
}

#[test]
fn test_gzip_zlib_limits() {
    let mut compressor = Compressor::new(1).unwrap();
    let mut decompressor = Decompressor::new();

    let original = vec![0u8; 1_000_000];

    // Zlib
    let compressed_zlib = compressor.compress_zlib(&original).unwrap();
    decompressor.set_max_memory_limit(500_000);
    let result = decompressor.decompress_zlib(&compressed_zlib, original.len());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

    // Reset limit
    decompressor.set_max_memory_limit(usize::MAX);
    // Set ratio limit
    decompressor.set_limit_ratio(10);
    let result = decompressor.decompress_zlib(&compressed_zlib, original.len());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

    // Gzip
    let compressed_gzip = compressor.compress_gzip(&original).unwrap();
    decompressor.set_max_memory_limit(500_000);
    let result = decompressor.decompress_gzip(&compressed_gzip, original.len());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

    decompressor.set_max_memory_limit(usize::MAX);
    decompressor.set_limit_ratio(10);
    let result = decompressor.decompress_gzip(&compressed_gzip, original.len());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
}
