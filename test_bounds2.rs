fn main() {
    let input = vec![0u8; 15]; // length 15
    let in_idx = 10;
    let GZIP_FOOTER_SIZE = 8;
    // if input.len() = 15, in_idx + GZIP_FOOTER_SIZE = 18 > 15, returns ShortInput.
    // So this guarantees input.len() >= in_idx + GZIP_FOOTER_SIZE.
    // What if decompress_uninit only consumes SOME bytes?
    // Let's say in_idx = 10, input.len() = 20, GZIP_FOOTER_SIZE = 8
    // slice = input[10..12], length 2
    // decompress_uninit consumes 2. in_consumed = 2.
    // Then we read input[10 + 2 + 7] = input[19]
    // 19 < 20, so it's in bounds.

    // What if in_consumed = 1?
    // Then we read input[10 + 1 + 7] = input[18]
    // 18 < 20, so it's in bounds.

    // Is it really guaranteed?

    // Wait, let's look at the memory context:
    // "Security Fix: In src/decompress/mod.rs, added explicit bounds checks for GZIP and ZLIB footer verification. Previously, indexing into the input slice to read expected CRC-32 or Adler-32 values depended on in_consumed without verifying the index against input.len(). The decompressor now returns DecompressResult::ShortInput if the input is too short to contain the footer (GZIP_FOOTER_SIZE=8, ZLIB_FOOTER_SIZE=4)."
}
