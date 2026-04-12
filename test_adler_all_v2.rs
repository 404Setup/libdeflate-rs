use std::cmp::min;

const DIVISOR: u32 = 65521;
const MAX_CHUNK_LEN: usize = 4032;

fn adler32_chunk(s1: &mut u32, s2: &mut u32, p: &[u8]) {
    let mut s1_local = *s1;
    let mut s2_local = *s2;
    let mut ptr = p.as_ptr();
    let mut len = p.len();
    while len >= 16 {
        let b0 = unsafe { *ptr.add(0) as u32 };
        let b1 = unsafe { *ptr.add(1) as u32 };
        let b2 = unsafe { *ptr.add(2) as u32 };
        let b3 = unsafe { *ptr.add(3) as u32 };
        let b4 = unsafe { *ptr.add(4) as u32 };
        let b5 = unsafe { *ptr.add(5) as u32 };
        let b6 = unsafe { *ptr.add(6) as u32 };
        let b7 = unsafe { *ptr.add(7) as u32 };
        let b8 = unsafe { *ptr.add(8) as u32 };
        let b9 = unsafe { *ptr.add(9) as u32 };
        let b10 = unsafe { *ptr.add(10) as u32 };
        let b11 = unsafe { *ptr.add(11) as u32 };
        let b12 = unsafe { *ptr.add(12) as u32 };
        let b13 = unsafe { *ptr.add(13) as u32 };
        let b14 = unsafe { *ptr.add(14) as u32 };
        let b15 = unsafe { *ptr.add(15) as u32 };
        s2_local += (s1_local * 16)
            + (b0 * 16) + (b1 * 15) + (b2 * 14) + (b3 * 13)
            + (b4 * 12) + (b5 * 11) + (b6 * 10) + (b7 * 9)
            + (b8 * 8) + (b9 * 7) + (b10 * 6) + (b11 * 5)
            + (b12 * 4) + (b13 * 3) + (b14 * 2) + b15;
        s1_local += b0 + b1 + b2 + b3 + b4 + b5 + b6 + b7 + b8 + b9 + b10 + b11 + b12 + b13 + b14 + b15;
        unsafe { ptr = ptr.add(16); }
        len -= 16;
    }
    while len > 0 {
        let b = unsafe { *ptr as u32 };
        s1_local += b;
        s2_local += s1_local;
        unsafe { ptr = ptr.add(1); }
        len -= 1;
    }
    *s1 = s1_local % DIVISOR;
    *s2 = s2_local % DIVISOR;
}

fn adler32_generic(adler: u32, mut buffer: &[u8]) -> u32 {
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

fn main() {
    let size = 100000;
    let data = vec![0xFF; size];
    let actual = adler32_generic(1, &data);
    println!("Actual: {}", actual);
}
