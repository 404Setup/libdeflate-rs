const DIVISOR: u32 = 65521;
fn main() {
    let mut s1: u32 = 65520;
    let mut s2: u32 = 65520;
    for _ in 0..32 {
        s1 += 255;
        s2 += s1;
    }
    println!("Ref s1: {}, s2: {}", s1 % DIVISOR, s2 % DIVISOR);

    let mut s1_avx2: u32 = 65520;
    let mut s2_avx2: u32 = 65520;
    let n = 32;
    s2_avx2 = ((s2_avx2 as u64 + s1_avx2 as u64 * n as u64) % DIVISOR as u64) as u32;
    let sum_s2: u64 = (1..=32).map(|i| (32-i+1)*255).sum();
    s2_avx2 = ((s2_avx2 as u64 + sum_s2) % DIVISOR as u64) as u32;
    let sum_s1: u64 = 32 * 255;
    s1_avx2 = ((s1_avx2 as u64 + sum_s1) % DIVISOR as u64) as u32;

    println!("AVX2 s1: {}, s2: {}", s1_avx2, s2_avx2);
}
