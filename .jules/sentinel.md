## 2025-02-18 - Unsafe Vector Folding in CRC32
**Vulnerability:** A specialized SIMD tail handling function `fold_lessthan16bytes_avx512` in `crc32/x86.rs` contained a potential out-of-bounds read (`p.offset(len - 16)` where `len < 16`) and performed incorrect folding calculations, leading to CRC mismatches for certain tail lengths.
**Learning:** Complexity in SIMD optimization for cold paths (like small tails) often introduces subtle correctness and security bugs that outweigh the negligible performance benefits.
**Prevention:** Prefer robust, simple scalar fallbacks for small tails and edge cases in SIMD algorithms. Avoid complex pointer arithmetic with negative offsets unless strictly verified safe.
