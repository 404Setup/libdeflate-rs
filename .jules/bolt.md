## 2024-05-24 - [Fallback Loop Optimization]
**Learning:** Porting SIMD optimizations from a primary fast loop to a secondary fallback loop yielded a 5x throughput improvement (1.2 GB/s -> 5.9 GB/s) for offset 9. However, for offset 8, the existing scalar unrolled loop (~11 GB/s) was slightly faster than the ported SIMD version (~8.9 GB/s), likely due to specific instruction scheduling or overheads.
**Action:** Always benchmark fallback paths when optimizing. Don't assume SIMD is automatically faster than well-unrolled scalar code for small constant offsets; verify each case.
