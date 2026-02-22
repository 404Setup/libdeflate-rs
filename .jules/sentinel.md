## 2025-02-18 - Unsafe Vector Folding in CRC32
**Vulnerability:** A specialized SIMD tail handling function `fold_lessthan16bytes_avx512` in `crc32/x86.rs` contained a potential out-of-bounds read (`p.offset(len - 16)` where `len < 16`) and performed incorrect folding calculations, leading to CRC mismatches for certain tail lengths.
**Learning:** Complexity in SIMD optimization for cold paths (like small tails) often introduces subtle correctness and security bugs that outweigh the negligible performance benefits.
**Prevention:** Prefer robust, simple scalar fallbacks for small tails and edge cases in SIMD algorithms. Avoid complex pointer arithmetic with negative offsets unless strictly verified safe.

## 2025-02-18 - Uninitialized Memory Read in prepare_pattern
**Vulnerability:** The `prepare_pattern` function in `src/decompress/mod.rs` was reading uninitialized memory from the output buffer for small offsets (3, 5, 6, 7). For example, offset 3 would read 4 bytes (u32), overlapping with the write cursor. Although the extra bytes were masked out, reading uninitialized memory is Undefined Behavior in Rust.
**Learning:** Optimizing small reads by over-reading and masking is dangerous when reading from a buffer that is being written to, as it may involve reading uninitialized memory. Safe Rust principles (avoiding UB) must take precedence over micro-optimizations that rely on specific compiler behaviors regarding uninitialized memory.
**Prevention:** Use composed reads (e.g., `u16` + `u8`) for odd-sized accesses or ensure the buffer is fully initialized if over-reading is necessary.
