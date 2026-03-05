use std::io::Read;
use std::io::Write;

fn generate_test_data(size: usize, pattern: u8) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if pattern == 0 {
        // All zeros
    } else if pattern == 1 {
        // Random uncompressible noise
        let mut state = 12345u32;
        for j in 0..size {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            data[j] = (state >> 24) as u8;
        }
    } else {
        // Minecraft-style mixed data
        for i in 0..size {
            if i % 100 < 5 {
                data[i] = i as u8;
            } else {
                data[i] = 0;
            }
        }
    }
    data
}

#[test]
fn test_deflate_interop() {
    let mut our_decompressor = libdeflate::api::Decompressor::new();

    let sizes = [0, 1, 10, 100, 1000, 10000, 100000];
    let patterns = [0, 1, 2];

    for &size in &sizes {
        for &pattern in &patterns {
            for level in 1..=9 {
                let mut our_compressor = libdeflate::api::Compressor::new(level).unwrap();
                let mut their_compressor = libdeflater::Compressor::new(libdeflater::CompressionLvl::new(level).unwrap());
                let mut their_decompressor = libdeflater::Decompressor::new();

                let data = generate_test_data(size, pattern);

                // 1. our compress -> their decompress
                let bound = our_compressor.deflate_compress_bound(data.len());
                let mut comp1 = vec![0u8; bound];
                let comp1_sz = our_compressor.compress_deflate_into(&data, &mut comp1).unwrap();
                comp1.truncate(comp1_sz);

                let mut decomp1 = vec![0u8; data.len()];
                if size > 0 {
                    let decomp_sz = their_decompressor.deflate_decompress(&comp1, &mut decomp1).unwrap();
                    assert_eq!(decomp_sz, data.len());
                    assert_eq!(decomp1, data);
                }

                // 2. their compress -> our decompress
                let their_bound = their_compressor.deflate_compress_bound(data.len());
                let mut comp2 = vec![0u8; their_bound];
                let comp2_sz = their_compressor.deflate_compress(&data, &mut comp2).unwrap();
                comp2.truncate(comp2_sz);

                let decomp2 = our_decompressor.decompress_deflate(&comp2, data.len()).unwrap();
                assert_eq!(decomp2, data);
            }
        }
    }
}

#[test]
fn test_zlib_interop() {
    let mut our_decompressor = libdeflate::api::Decompressor::new();

    let sizes = [0, 1, 10, 100, 1000, 10000, 100000];
    let patterns = [0, 1, 2];

    for &size in &sizes {
        for &pattern in &patterns {
            for level in 1..=9 {
                let mut our_compressor = libdeflate::api::Compressor::new(level).unwrap();
                let mut their_compressor = libdeflater::Compressor::new(libdeflater::CompressionLvl::new(level).unwrap());
                let mut their_decompressor = libdeflater::Decompressor::new();

                let data = generate_test_data(size, pattern);

                // 1. our compress -> their decompress
                let bound = our_compressor.zlib_compress_bound(data.len());
                let mut comp1 = vec![0u8; bound];
                let comp1_sz = our_compressor.compress_zlib_into(&data, &mut comp1).unwrap();
                comp1.truncate(comp1_sz);

                let mut decomp1 = vec![0u8; data.len()];
                if size > 0 {
                    let decomp_sz = their_decompressor.zlib_decompress(&comp1, &mut decomp1).unwrap();
                    assert_eq!(decomp_sz, data.len());
                    assert_eq!(decomp1, data);
                }

                // 2. their compress -> our decompress
                let their_bound = their_compressor.zlib_compress_bound(data.len());
                let mut comp2 = vec![0u8; their_bound];
                let comp2_sz = their_compressor.zlib_compress(&data, &mut comp2).unwrap();
                comp2.truncate(comp2_sz);

                let decomp2 = our_decompressor.decompress_zlib(&comp2, data.len()).unwrap();
                assert_eq!(decomp2, data);
            }
        }
    }
}

#[test]
fn test_gzip_interop() {
    let mut our_decompressor = libdeflate::api::Decompressor::new();

    let sizes = [0, 1, 10, 100, 1000, 10000, 100000];
    let patterns = [0, 1, 2];

    for &size in &sizes {
        for &pattern in &patterns {
            for level in 1..=9 {
                let mut our_compressor = libdeflate::api::Compressor::new(level).unwrap();
                let mut their_compressor = libdeflater::Compressor::new(libdeflater::CompressionLvl::new(level).unwrap());
                let mut their_decompressor = libdeflater::Decompressor::new();

                let data = generate_test_data(size, pattern);

                // 1. our compress -> their decompress
                let bound = our_compressor.gzip_compress_bound(data.len());
                let mut comp1 = vec![0u8; bound];
                let comp1_sz = our_compressor.compress_gzip_into(&data, &mut comp1).unwrap();
                comp1.truncate(comp1_sz);

                let mut decomp1 = vec![0u8; data.len()];
                if size > 0 {
                    let decomp_sz = their_decompressor.gzip_decompress(&comp1, &mut decomp1).unwrap();
                    assert_eq!(decomp_sz, data.len());
                    assert_eq!(decomp1, data);
                }

                // 2. their compress -> our decompress
                let their_bound = their_compressor.gzip_compress_bound(data.len());
                let mut comp2 = vec![0u8; their_bound];
                let comp2_sz = their_compressor.gzip_compress(&data, &mut comp2).unwrap();
                comp2.truncate(comp2_sz);

                let decomp2 = our_decompressor.decompress_gzip(&comp2, data.len()).unwrap();
                assert_eq!(decomp2, data);
            }
        }
    }
}
