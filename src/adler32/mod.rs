use std::cmp::min;

const DIVISOR: u32 = 65521;
const MAX_CHUNK_LEN: usize = 5552;

#[inline]
fn adler32_chunk(s1: &mut u32, s2: &mut u32, mut p: &[u8]) {
    let mut n = p.len();
    let mut s1_local = *s1;
    let mut s2_local = *s2;

    while n >= 16 {
        let b0 = p[0] as u32;
        let b1 = p[1] as u32;
        let b2 = p[2] as u32;
        let b3 = p[3] as u32;
        let b4 = p[4] as u32;
        let b5 = p[5] as u32;
        let b6 = p[6] as u32;
        let b7 = p[7] as u32;
        let b8 = p[8] as u32;
        let b9 = p[9] as u32;
        let b10 = p[10] as u32;
        let b11 = p[11] as u32;
        let b12 = p[12] as u32;
        let b13 = p[13] as u32;
        let b14 = p[14] as u32;
        let b15 = p[15] as u32;

        s2_local += (s1_local * 16)
            + (b0 * 16)
            + (b1 * 15)
            + (b2 * 14)
            + (b3 * 13)
            + (b4 * 12)
            + (b5 * 11)
            + (b6 * 10)
            + (b7 * 9)
            + (b8 * 8)
            + (b9 * 7)
            + (b10 * 6)
            + (b11 * 5)
            + (b12 * 4)
            + (b13 * 3)
            + (b14 * 2)
            + (b15 * 1);

        s1_local += b0 + b1 + b2 + b3 + b4 + b5 + b6 + b7 + b8 + b9 + b10 + b11 + b12 + b13 + b14 + b15;

        p = &p[16..];
        n -= 16;
    }

    while n >= 4 {
        let b0 = p[0] as u32;
        let b1 = p[1] as u32;
        let b2 = p[2] as u32;
        let b3 = p[3] as u32;

        s2_local += (s1_local * 4) + (b0 * 4) + (b1 * 3) + (b2 * 2) + (b3 * 1);
        s1_local += b0 + b1 + b2 + b3;

        p = &p[4..];
        n -= 4;
    }

    for &b in p {
        s1_local += b as u32;
        s2_local += s1_local;
    }

    *s1 = s1_local % DIVISOR;
    *s2 = s2_local % DIVISOR;
}

pub fn adler32_generic(adler: u32, mut buffer: &[u8]) -> u32 {
    let mut s1 = adler & 0xFFFF;
    let mut s2 = adler >> 16;
    let mut len = buffer.len();

    while len > 0 {
        let n = min(len, MAX_CHUNK_LEN);
        let (chunk, rest) = buffer.split_at(n);
        buffer = rest;
        len -= n;

        adler32_chunk(&mut s1, &mut s2, chunk);
    }

    (s2 % DIVISOR) << 16 | (s1 % DIVISOR)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
mod arm;

pub fn adler32(adler: u32, slice: &[u8]) -> u32 {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx512vnni") {
            return unsafe { x86::adler32_x86_avx512_vnni(adler, slice) };
        }
        if is_x86_feature_detected!("avxvnni") {
            return unsafe { x86::adler32_x86_avx2_vnni(adler, slice) };
        }
        if is_x86_feature_detected!("avx2") {
            return unsafe { x86::adler32_x86_avx2(adler, slice) };
        }
        if is_x86_feature_detected!("sse2") {
            return unsafe { x86::adler32_x86_sse2(adler, slice) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("dotprod") {
            return unsafe { arm::adler32_arm_neon_dotprod(adler, slice) };
        }
        if std::arch::is_aarch64_feature_detected!("neon") {
            return unsafe { arm::adler32_arm_neon(adler, slice) };
        }
    }

    adler32_generic(adler, slice)
}