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
                    if buffer.try_reserve(bound).is_err() {
                        return Vec::new();
                    }
                    let buf_slice = &mut buffer.spare_capacity_mut()[..bound];

                    let (res, size, _) =
                        compressor.compress(input, buf_slice, crate::compress::FlushMode::Finish);
                    if res == CompressResult::Success {
                        assert!(size <= bound);
                        unsafe {
                            buffer.set_len(size);
                        }
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

    /// Returns one result per input. Missing size limits produce `None`;
    /// extra size limits are ignored.
    pub fn decompress_batch(
        &self,
        inputs: &[&[u8]],
        max_out_sizes: &[usize],
    ) -> Vec<Option<Vec<u8>>> {
        inputs
            .par_iter()
            .enumerate()
            .map_init(
                || (Decompressor::new(), Vec::new()),
                |(decompressor, buffer), (i, &input)| {
                    let &max_size = max_out_sizes.get(i)?;
                    buffer.clear();
                    buffer.try_reserve(max_size).ok()?;
                    let buf_slice = &mut buffer.spare_capacity_mut()[..max_size];

                    let (res, _, size) =
                        unsafe { decompressor.decompress_uninit(input, buf_slice) };
                    if res == DecompressResult::Success {
                        assert!(size <= max_size);
                        unsafe {
                            buffer.set_len(size);
                        }
                        Some(std::mem::take(buffer))
                    } else {
                        None
                    }
                },
            )
            .collect()
    }
}
