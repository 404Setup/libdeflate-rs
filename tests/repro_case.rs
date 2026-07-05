use libdeflate::{Compressor, Decompressor};

fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn gen_text(seed: u64, len: usize) -> Vec<u8> {
    let mut s = seed | 1;
    let words = [b"hello ".as_ref(), b"world ", b"deflate ", b"rust "];
    let mut v = Vec::with_capacity(len);
    while v.len() < len {
        let w = words[(xorshift(&mut s) % 4) as usize];
        v.extend_from_slice(&w[..w.len().min(len - v.len())]);
    }
    v
}

#[test]
fn repro_lvl3_300k() {
    let data = gen_text(3 * 1000 + 11 * 10 + 2, 300_000);
    let mut comp = Compressor::new(3).unwrap();
    let mut cbuf = vec![0u8; comp.deflate_compress_bound(data.len())];
    let clen = comp.compress_deflate_into(&data, &mut cbuf).unwrap();
    println!("[DEBUG_LOG] clen={}", clen);

    let mut ref_d = libdeflater::Decompressor::new();
    let mut rbuf = vec![0u8; data.len()];
    match ref_d.deflate_decompress(&cbuf[..clen], &mut rbuf) {
        Ok(n) => {
            println!("[DEBUG_LOG] libdeflater ok n={}", n);
            assert_eq!(&rbuf[..n], &data[..], "libdeflater output mismatch");
        }
        Err(e) => println!("[DEBUG_LOG] libdeflater err={:?} => our COMPRESSOR is buggy", e),
    }

    let mut d = Decompressor::new();
    let mut dbuf = vec![0u8; data.len()];
    match d.decompress_deflate_into(&cbuf[..clen], &mut dbuf) {
        Ok(n) => println!("[DEBUG_LOG] ours ok n={}", n),
        Err(e) => println!("[DEBUG_LOG] ours err={:?}", e),
    }
}
