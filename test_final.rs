const DIVISOR: u32 = 65521;
fn main() {
    let mut s1: u32 = 65520;
    let mut s2: u32 = 65520;

    // Process 64 bytes of 0xFF
    for _ in 0..64 {
        s1 += 255;
        s2 += s1;
    }
    println!("Ref s1: {}, s2: {}", s1 % DIVISOR, s2 % DIVISOR);

    // SSE2 scalar tail part:
    let mut s1_sse2: u32 = 65520;
    let mut s2_sse2: u32 = 65520;
    let n = 64;
    // s2 = ((s2 as u64 + s1 as u64 * n as u64) % DIVISOR as u64) as u32;
    s2_sse2 = ((s2_sse2 as u64 + s1_sse2 as u64 * n as u64) % DIVISOR as u64) as u32;
    // sum_s2 = sum( (n-i+1)*bi ) = sum( (64-i+1)*255 )
    let sum_s2: u64 = (1..=64).map(|i| (64-i+1)*255).sum();
    s2_sse2 = ((s2_sse2 as u64 + sum_s2) % DIVISOR as u64) as u32;

    let sum_s1: u64 = 64 * 255;
    s1_sse2 = ((s1_sse2 as u64 + sum_s1) % DIVISOR as u64) as u32;

    println!("SSE2 s1: {}, s2: {}", s1_sse2, s2_sse2);
}
