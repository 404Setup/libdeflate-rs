use libdeflate::{Compressor, Decompressor};

fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn gen_data(seed: u64, len: usize, kind: u32) -> Vec<u8> {
    let mut s = seed | 1;
    let mut v = Vec::with_capacity(len);
    match kind {
        0 => {
            // random
            while v.len() < len {
                v.push((xorshift(&mut s) & 0xFF) as u8);
            }
        }
        1 => {
            let pat_len = 1 + (xorshift(&mut s) % 64) as usize;
            let pat: Vec<u8> = (0..pat_len)
                .map(|_| (xorshift(&mut s) & 0x0F) as u8)
                .collect();
            while v.len() < len {
                v.extend_from_slice(&pat[..pat_len.min(len - v.len())]);
            }
        }
        2 => {
            let words = [b"hello ".as_ref(), b"world ", b"deflate ", b"rust "];
            while v.len() < len {
                let w = words[(xorshift(&mut s) % 4) as usize];
                v.extend_from_slice(&w[..w.len().min(len - v.len())]);
            }
        }
        _ => {
            while v.len() < len {
                if xorshift(&mut s) & 1 == 0 {
                    v.push(0xAA);
                } else {
                    v.push((xorshift(&mut s) & 0xFF) as u8);
                }
            }
        }
    }
    v
}

#[test]
fn fuzz_roundtrip_self_and_interop() {
    let sizes = [0usize, 1, 2, 3, 4, 7, 32, 255, 256, 4095, 65536, 300_000];
    for level in [0, 1, 3, 6, 9, 12] {
        let mut comp = Compressor::new(level).unwrap();
        let mut decomp = Decompressor::new();
        for (i, &size) in sizes.iter().enumerate() {
            for kind in 0..4 {
                let data = gen_data(
                    (level as u64) * 1000 + i as u64 * 10 + kind as u64,
                    size,
                    kind,
                );
                let bound = comp.deflate_compress_bound(data.len());
                let mut cbuf = vec![0u8; bound];
                let clen = comp.compress_deflate_into(&data, &mut cbuf).unwrap_or_else(|e| panic!("comp fail lvl={} size={} kind={} bound={} err={:?}", level, size, kind, bound, e));
                let mut dbuf = vec![0u8; data.len()];
                let dlen = decomp.decompress_deflate_into(&cbuf[..clen], &mut dbuf).unwrap_or_else(|e| panic!("decomp fail lvl={} size={} kind={} clen={} err={:?}", level, size, kind, clen, e));
                assert_eq!(dlen, data.len(), "deflate len lvl={} size={} kind={}", level, size, kind);
                assert_eq!(dbuf, data, "deflate data lvl={} size={} kind={}", level, size, kind);

                let mut ref_d = libdeflater::Decompressor::new();
                let mut rbuf = vec![0u8; data.len()];
                let rlen = ref_d
                    .deflate_decompress(&cbuf[..clen], &mut rbuf)
                    .expect("libdeflater failed to decompress our output");
                assert_eq!(rlen, data.len());
                assert_eq!(rbuf, data, "interop c->ref lvl={} size={} kind={}", level, size, kind);

                let mut ref_c =
                    libdeflater::Compressor::new(libdeflater::CompressionLvl::new(6).unwrap());
                let mut cbuf2 = vec![0u8; ref_c.deflate_compress_bound(data.len())];
                let clen2 = ref_c.deflate_compress(&data, &mut cbuf2).unwrap();
                let mut dbuf2 = vec![0u8; data.len()];
                let dlen2 = decomp
                    .decompress_deflate_into(&cbuf2[..clen2], &mut dbuf2)
                    .expect("our decompressor failed on libdeflater output");
                assert_eq!(dlen2, data.len());
                assert_eq!(dbuf2, data, "interop ref->d lvl={} size={} kind={}", level, size, kind);

                let zbound = comp.zlib_compress_bound(data.len());
                let mut zbuf = vec![0u8; zbound];
                let zlen = comp.compress_zlib_into(&data, &mut zbuf).unwrap();
                let mut zd = vec![0u8; data.len()];
                let zdlen = decomp.decompress_zlib_into(&zbuf[..zlen], &mut zd).unwrap();
                assert_eq!(zdlen, data.len());
                assert_eq!(zd, data, "zlib lvl={} size={} kind={}", level, size, kind);

                let gbound = comp.gzip_compress_bound(data.len());
                let mut gbuf = vec![0u8; gbound];
                let glen = comp.compress_gzip_into(&data, &mut gbuf).unwrap();
                let mut gd = vec![0u8; data.len()];
                let gdlen = decomp.decompress_gzip_into(&gbuf[..glen], &mut gd).unwrap();
                assert_eq!(gdlen, data.len());
                assert_eq!(gd, data, "gzip lvl={} size={} kind={}", level, size, kind);
            }
        }
    }
}

#[test]
fn fuzz_garbage_input_no_panic() {
    let mut decomp = Decompressor::new();
    for seed in 0..200u64 {
        let data = gen_data(seed + 999, 512, 0);
        let mut out = vec![0u8; 4096];
        let _ = decomp.decompress_deflate_into(&data, &mut out);
        let _ = decomp.decompress_zlib_into(&data, &mut out);
        let _ = decomp.decompress_gzip_into(&data, &mut out);
    }
}

#[test]
fn fuzz_truncated_valid_stream_no_panic() {
    let mut comp = Compressor::new(6).unwrap();
    let mut decomp = Decompressor::new();
    let data = gen_data(42, 10_000, 2);
    let mut cbuf = vec![0u8; comp.deflate_compress_bound(data.len())];
    let clen = comp.compress_deflate_into(&data, &mut cbuf).unwrap();
    for cut in 0..clen.min(300) {
        let mut out = vec![0u8; data.len()];
        let _ = decomp.decompress_deflate_into(&cbuf[..cut], &mut out);
    }
}




