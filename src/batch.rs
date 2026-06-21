use crate::compress::{CompressResult, Compressor};
use crate::decompress::{DecompressResult, Decompressor};
use rayon::prelude::*;

pub struct BatchCompressor {
    level: usize,
}

impl BatchCompressor {
    pub fn new(level: usize) -> Self {
        Self { level }
    }

    pub fn compress_batch(&self, inputs: &[&[u8]]) -> Vec<Vec<u8>> {
        inputs
            .par_iter()
            .map_init(
                || (Compressor::new(self.level), Vec::new()),
                |(compressor, buffer), &input| {
                    let bound = Compressor::deflate_compress_bound(input.len());
                    buffer.clear();
                    buffer.reserve(bound);
                    unsafe {
                        buffer.set_len(bound);
                    }
                    let buf_slice = crate::common::slice_as_uninit_mut(&mut buffer[..bound]);

                    let (res, size, _) =
                        compressor.compress(input, buf_slice, crate::compress::FlushMode::Finish);
                    if res == CompressResult::Success {
                        assert!(size <= bound);
                        buffer.truncate(size);
                        std::mem::take(buffer)
                    } else {
                        Vec::new()
                    }
                },
            )
            .collect()
    }
}

pub struct BatchDecompressor;

crate::impl_default_new!(BatchDecompressor);

impl BatchDecompressor {
    pub fn new() -> Self {
        Self
    }

    pub fn decompress_batch(
        &self,
        inputs: &[&[u8]],
        max_out_sizes: &[usize],
    ) -> Vec<Option<Vec<u8>>> {
        inputs
            .par_iter()
            .zip(max_out_sizes.par_iter())
            .map_init(
                || (Decompressor::new(), Vec::new()),
                |(decompressor, buffer), (&input, &max_size)| {
                    buffer.clear();
                    buffer.reserve(max_size);
                    unsafe {
                        buffer.set_len(max_size);
                    }
                    let buf_slice = crate::common::slice_as_uninit_mut(&mut buffer[..max_size]);

                    let (res, _, size) =
                        unsafe { decompressor.decompress_uninit(input, buf_slice) };
                    if res == DecompressResult::Success {
                        assert!(size <= max_size);
                        buffer.truncate(size);
                        Some(std::mem::take(buffer))
                    } else {
                        None
                    }
                },
            )
            .collect()
    }
}
