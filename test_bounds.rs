fn main() {
    // wait.
    // If I add `if in_idx + in_consumed + GZIP_FOOTER_SIZE > input.len() { return (DecompressResult::ShortInput, ...); }`
    // Will it ever trigger?
    // Let's think if `in_consumed` COULD exceed the sub-slice length!
    // In `decompress_bmi2_ptr`:
    // `in_idx` is returned.
    // `in_idx` might increment by 7 in `refill_bits!`.
    // BUT only if `in_len - in_idx >= 8`.
    // So `in_idx + 7 <= in_len - 1 < in_len`.
    // So `in_idx` NEVER exceeds `in_len`!
    // What if `decompress_uninit` HAS A BUG where it returns `in_consumed` > `in_len`?
    // IF SO, it is a huge vulnerability anyway.

    // SO, the reporter is just doing manual review and wants me to add an EXPLICIT bounds check for the footer.
    // I will add:
    // `if in_idx + in_consumed + GZIP_FOOTER_SIZE > input.len() { return (DecompressResult::ShortInput, 0, 0); }`
    // right after `if res != DecompressResult::Success { ... }`
    // AND ALSO for ZLIB:
    // `if 2 + in_consumed + ZLIB_FOOTER_SIZE > input.len() { return (DecompressResult::ShortInput, 0, 0); }`
}
