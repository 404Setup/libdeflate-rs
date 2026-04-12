use std::cmp::min;

const DIVISOR: u32 = 65521;
const MAX_CHUNK_LEN: usize = 4032;

fn adler32_chunk(s1: &mut u32, s2: &mut u32, p: &[u8]) {
    let mut s1_local = *s1;
    let mut s2_local = *s2;
    for &b in p {
        s1_local += b as u32;
        s2_local += s1_local;
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
    let expected = adler32_generic(1, &data);
    println!("Expected: {}", expected);
}
