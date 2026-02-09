use crate::compress::{CompressResult, Compressor as InternalCompressor, FlushMode};
use crate::decompress::Decompressor as InternalDecompressor;
use std::io::{self};

pub struct Compressor {
    inner: InternalCompressor,
}

impl Compressor {
    pub fn new(level: i32) -> io::Result<Self> {
        if level < 0 || level > 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Compression level must be between 0 and 12",
            ));
        }
        Ok(Self {
            inner: InternalCompressor::new(level as usize),
        })
    }

    pub fn compress_deflate(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        let bound = self.deflate_compress_bound(data.len());
        self.compress_helper(data, bound, |c, data, out| {
            let (res, size, _) = c.compress(data, out, FlushMode::Finish);
            (res, size)
        })
    }

    pub fn compress_deflate_into(&mut self, data: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let (res, size, _) = self.inner.compress(data, output, FlushMode::Finish);
        if res == CompressResult::Success {
            Ok(size)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Insufficient space"))
        }
    }

    pub fn compress_zlib(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        let bound = self.zlib_compress_bound(data.len());
        self.compress_helper(data, bound, |c, data, out| c.compress_zlib(data, out))
    }

    pub fn compress_zlib_into(&mut self, data: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let (res, size) = self.inner.compress_zlib(data, output);
        if res == CompressResult::Success {
            Ok(size)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Compression failed"))
        }
    }

    pub fn compress_gzip(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        let bound = self.gzip_compress_bound(data.len());
        self.compress_helper(data, bound, |c, data, out| c.compress_gzip(data, out))
    }

    pub fn compress_gzip_into(&mut self, data: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let (res, size) = self.inner.compress_gzip(data, output);
        if res == CompressResult::Success {
            Ok(size)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Compression failed"))
        }
    }

    pub fn deflate_compress_bound(&mut self, size: usize) -> usize {
        InternalCompressor::deflate_compress_bound(size)
    }

    pub fn zlib_compress_bound(&mut self, size: usize) -> usize {
        InternalCompressor::zlib_compress_bound(size)
    }

    pub fn gzip_compress_bound(&mut self, size: usize) -> usize {
        InternalCompressor::gzip_compress_bound(size)
    }

    fn compress_helper<F>(&mut self, data: &[u8], bound: usize, f: F) -> io::Result<Vec<u8>>
    where
        F: FnOnce(&mut InternalCompressor, &[u8], &mut [u8]) -> (CompressResult, usize),
    {
        let mut output = Vec::with_capacity(bound);
        unsafe {
            output.set_len(bound);
        }
        let (res, size) = f(&mut self.inner, data, &mut output);
        match res {
            CompressResult::Success => {
                unsafe {
                    output.set_len(size);
                }
                Ok(output)
            }
            CompressResult::InsufficientSpace => {
                Err(io::Error::new(io::ErrorKind::Other, "Insufficient space"))
            }
        }
    }
}

pub struct Decompressor {
    inner: InternalDecompressor,
}

impl Decompressor {
    pub fn new() -> Self {
        Self {
            inner: InternalDecompressor::new(),
        }
    }

    pub fn decompress_deflate(&mut self, data: &[u8], expected_size: usize) -> io::Result<Vec<u8>> {
        self.decompress_helper(data, expected_size, |d, data, out| d.decompress(data, out))
    }

    pub fn decompress_deflate_into(&mut self, data: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let (res, _, size) = self.inner.decompress(data, output);
        if res == crate::decompress::DecompressResult::Success {
            Ok(size)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decompression failed",
            ))
        }
    }

    pub fn decompress_zlib(&mut self, data: &[u8], expected_size: usize) -> io::Result<Vec<u8>> {
        self.decompress_helper(data, expected_size, |d, data, out| {
            d.decompress_zlib(data, out)
        })
    }

    pub fn decompress_zlib_into(&mut self, data: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let (res, _, size) = self.inner.decompress_zlib(data, output);
        if res == crate::decompress::DecompressResult::Success {
            Ok(size)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decompression failed",
            ))
        }
    }

    pub fn decompress_gzip(&mut self, data: &[u8], expected_size: usize) -> io::Result<Vec<u8>> {
        self.decompress_helper(data, expected_size, |d, data, out| {
            d.decompress_gzip(data, out)
        })
    }

    pub fn decompress_gzip_into(&mut self, data: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let (res, _, size) = self.inner.decompress_gzip(data, output);
        if res == crate::decompress::DecompressResult::Success {
            Ok(size)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decompression failed",
            ))
        }
    }

    fn decompress_helper<F>(
        &mut self,
        data: &[u8],
        expected_size: usize,
        f: F,
    ) -> io::Result<Vec<u8>>
    where
        F: FnOnce(
            &mut InternalDecompressor,
            &[u8],
            &mut [u8],
        ) -> (crate::decompress::DecompressResult, usize, usize),
    {
        let mut output = Vec::with_capacity(expected_size);
        unsafe {
            output.set_len(expected_size);
        }
        let (res, _, size) = f(&mut self.inner, data, &mut output);
        if res == crate::decompress::DecompressResult::Success {
            output.truncate(size);
            Ok(output)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decompression failed",
            ))
        }
    }
}
