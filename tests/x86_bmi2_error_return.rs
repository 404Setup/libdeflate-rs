use libdeflate::decompress::{Decompressor, DecompressResult};
use libdeflate::compress::Compressor;
use libdeflate::compress::FlushMode;
use std::mem::MaybeUninit;

#[test]
fn test_x86_bmi2_error_preserves_progress() {
    let mut decompressor = Decompressor::new();
    let mut compressor = Compressor::new(1);

    // Create a valid block
    let mut out: [MaybeUninit<u8>; 100] = [MaybeUninit::uninit(); 100];
    let (res, size, _) = compressor.compress(b"ABC", &mut out, FlushMode::Sync);
    assert_eq!(res, libdeflate::compress::CompressResult::Success);

    // extract compressed data
    let mut input: Vec<u8> = out[..size].iter().map(|x| unsafe { x.assume_init() }).collect();

    // The valid block is successfully decompressed. Now append an invalid block.
    // Dynamic Huffman block with missing data
    input.push(0b00000101); // Final block, dynamic huffman
    input.push(0); // just some missing data for dynamic header to fail

    let mut dec_out: [MaybeUninit<u8>; 20] = [MaybeUninit::uninit(); 20];
    unsafe {
        let (res, in_cons, out_prod) = decompressor.decompress_uninit(&input, &mut dec_out);
        println!("res: {:?}, in_cons: {}, out_prod: {}", res, in_cons, out_prod);

        // Should return a failure (ShortInput or BadData)
        assert!(res != DecompressResult::Success);

        // Progress should be preserved!
        assert_eq!(out_prod, 3, "out_prod should not be 0 after successful blocks");
        assert!(in_cons > 0, "in_cons should not be 0 after successful blocks");
    }
}

#[test]
fn test_x86_bmi2_uncompressed_error_preserves_progress() {
    let mut decompressor = Decompressor::new();

    // First block: uncompressed, Valid, length 3
    let mut input = vec![0b00000000]; // Not final block, uncompressed
    input.extend_from_slice(&[0x03, 0x00, 0xFC, 0xFF]); // len=3, nlen=!len
    input.extend_from_slice(&[0x41, 0x42, 0x43]); // "ABC"

    // 2nd block
    input.push(0b00000000); // Not final block, uncompressed
    input.extend_from_slice(&[0x10, 0x00, 0xEF, 0xFF]); // 16 bytes
    input.extend_from_slice(&[0x41, 0x42]); // Only 2 bytes instead of 16

    let mut out: [MaybeUninit<u8>; 20] = [MaybeUninit::uninit(); 20];
    unsafe {
        let (res, in_cons, out_prod) = decompressor.decompress_uninit(&input, &mut out);
        println!("res: {:?}, in_cons: {}, out_prod: {}", res, in_cons, out_prod);

        // Before the fix, this would return (BadData, 0, 0)
        assert_eq!(res, DecompressResult::BadData);
        assert_eq!(out_prod, 3, "out_prod should not be 0 after successful blocks");
        assert!(in_cons > 0, "in_cons should not be 0 after successful blocks");
    }
}
