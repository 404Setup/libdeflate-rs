use libdeflate::decompress::{DecompressResult, Decompressor};

// Encode a dynamic header with a complete precode supporting all 19 symbols.
fn dynamic_header(lengths: &[u8], repeat: Option<u8>) -> Vec<u8> {
    let mut bits = Vec::new();
    let mut write = |value: u32, count: u32| {
        bits.extend((0..count).map(|i| ((value >> i) & 1) as u8));
    };
    write(5, 3); // Final dynamic block.
    write(0, 5); // 257 literal/length symbols.
    write(0, 5); // One distance symbol.
    write(15, 4); // All 19 precode lengths.
    for sym in [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ] {
        write(if sym < 13 { 4 } else { 5 }, 3);
    }
    for sym in lengths.iter().copied().chain(repeat) {
        let (code, len) = if sym < 13 {
            (sym as u32, 4)
        } else {
            (sym as u32 + 13, 5)
        };
        write(code.reverse_bits() >> (32 - len), len);
    }
    if let Some(sym) = repeat {
        write(
            0,
            match sym {
                16 => 2,
                17 => 3,
                18 => 7,
                _ => unreachable!(),
            },
        );
    }
    write(0, 1); // End-of-block for the single-symbol literal/length tree.
    bits.chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0, |byte, (i, bit)| byte | (bit << i))
        })
        .collect()
}

#[test]
fn rejects_code_length_repeats_past_header_end() {
    let mut lengths = vec![0; 257];
    lengths[256] = 1;
    for repeat in [16, 17, 18] {
        let input = dynamic_header(&lengths, Some(repeat));
        assert!(
            libdeflater::Decompressor::new()
                .deflate_decompress(&input, &mut [])
                .is_err()
        );
        assert_eq!(
            Decompressor::new().decompress(&input, &mut []).0,
            DecompressResult::BadData
        );
        assert_eq!(
            Decompressor::new()
                .decompress_streaming(&input, &mut [], &mut 0)
                .0,
            DecompressResult::BadData
        );
    }
    lengths.push(0);
    let valid = dynamic_header(&lengths, None);
    assert_eq!(
        Decompressor::new().decompress(&valid, &mut []).0,
        DecompressResult::Success
    );
    assert_eq!(
        libdeflater::Decompressor::new()
            .deflate_decompress(&valid, &mut [])
            .unwrap(),
        0
    );
}

#[test]
fn failed_dynamic_header_does_not_poison_static_tables() {
    let data = vec![b'a'; 1024];
    let fixed = libdeflate::Compressor::new(1)
        .unwrap()
        .compress_deflate(&data)
        .unwrap();
    assert_eq!((fixed[0] >> 1) & 3, 1);
    let mut decoder = Decompressor::new();
    let mut output = vec![0; data.len()];
    assert_eq!(
        decoder.decompress(&fixed, &mut output).0,
        DecompressResult::Success
    );
    let mut lengths = vec![0; 258];
    for i in [0, 1, 2, 256, 257] {
        lengths[i] = 1;
    }
    let invalid = dynamic_header(&lengths, None);
    assert_eq!(
        decoder.decompress(&invalid, &mut output).0,
        DecompressResult::BadData
    );
    assert_eq!(
        decoder.decompress(&fixed, &mut output).0,
        DecompressResult::Success
    );
    assert_eq!(output, data);
}

#[test]
fn streaming_preserves_prefetched_stored_header_and_body() {
    // Fixed + stored, stored with prefetched payload, and consecutive stored blocks.
    for (input, expected) in [
        (&[0x4a, 0x04, 0x04, 1, 0, 0xfe, 0xff, b'b'][..], &b"ab"[..]),
        (&[1, 3, 0, 0xfc, 0xff, b'a', b'b', b'c'], &b"abc"[..]),
        (
            &[
                0x4a, 0x04, 0, 1, 0, 0xfe, 0xff, b'b', 1, 1, 0, 0xfe, 0xff, b'c',
            ],
            &b"abc"[..],
        ),
    ] {
        let mut output = vec![0; expected.len()];
        assert_eq!(
            libdeflater::Decompressor::new()
                .deflate_decompress(input, &mut output)
                .unwrap(),
            expected.len()
        );
        assert_eq!(output, expected);
        for split in 0..=input.len() {
            for first_size in 0..=expected.len() {
                output.fill(0);
                let mut decoder = Decompressor::new();
                let mut written = 0;
                let (result, consumed, _) = decoder.decompress_streaming(
                    &input[..split],
                    &mut output[..first_size],
                    &mut written,
                );
                assert_ne!(result, DecompressResult::BadData);
                let (result, _, _) =
                    decoder.decompress_streaming(&input[consumed..], &mut output, &mut written);
                assert_eq!(
                    result,
                    DecompressResult::Success,
                    "split={split}, first_size={first_size}"
                );
                assert_eq!(written, expected.len());
                assert_eq!(output, expected);
            }
        }
    }
}

#[test]
fn streaming_rejects_output_position_past_buffer() {
    let mut decoder = Decompressor::new();
    let mut position = 1;
    assert_eq!(
        decoder.decompress_streaming(&[1, 0, 0, 255, 255], &mut [], &mut position),
        (DecompressResult::InsufficientSpace, 0, 0)
    );
    assert_eq!(position, 1);
}

#[test]
fn one_shot_reports_stream_end_before_trailing_bytes() {
    let data: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
    for level in [0, 1, 6] {
        let mut compressor = libdeflate::Compressor::new(level).unwrap();
        for format in 0..3 {
            let mut input = match format {
                0 => compressor.compress_deflate(&data),
                1 => compressor.compress_zlib(&data),
                _ => compressor.compress_gzip(&data),
            }
            .unwrap();
            let compressed_size = input.len();
            input.extend_from_slice(&[0xa5; 32]);
            let mut decoder = Decompressor::new();
            let mut output = vec![0; data.len()];
            let (result, consumed, produced) = match format {
                0 => decoder.decompress(&input, &mut output),
                1 => decoder.decompress_zlib(&input, &mut output),
                _ => decoder.decompress_gzip(&input, &mut output),
            };
            assert_eq!(
                result,
                DecompressResult::Success,
                "level={level}, format={format}"
            );
            assert_eq!(consumed, compressed_size, "level={level}, format={format}");
            assert_eq!(produced, data.len());
            assert_eq!(output, data);
        }
    }
}
