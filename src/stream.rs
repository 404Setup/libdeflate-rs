use crate::compress::{CompressResult, Compressor};
use crate::decompress::{DecompressResult, Decompressor, DecompressorState};
use rayon::prelude::*;
use std::cmp::min;
use std::io::{self, Read, Write};

/// A streaming encoder that compresses data using the DEFLATE algorithm.
///
/// # Warning
///
/// If the encoder is dropped without calling [`finish()`](Self::finish), the internal buffer will be
/// flushed, but any I/O errors that occur during this process will be silently ignored.
/// To ensure data integrity and handle errors, always call `finish()` explicitly.
/// An error while compressing or writing a buffered block makes the encoder unusable.
pub struct DeflateEncoder<W: Write + Send> {
    writer: Option<W>,
    buffer: Vec<u8>,
    buffer_size: usize,
    level: usize,
    compressors: Vec<Compressor>,
    output_buffers: Vec<Vec<u8>>,
    failed: bool,
}

impl<W: Write + Send> DeflateEncoder<W> {
    pub fn new(writer: W, level: usize) -> Self {
        Self {
            writer: Some(writer),
            buffer: Vec::with_capacity(1024 * 1024),
            buffer_size: 1024 * 1024,
            level,
            compressors: Vec::new(),
            output_buffers: Vec::new(),
            failed: false,
        }
    }

    /// Limits buffered input bytes. A size of zero is treated as one byte.
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size.max(1);
        self.buffer.reserve(size);
        self
    }

    fn flush_buffer_parallel(
        &mut self,
        final_block: bool,
        chunk_size: usize,
        buffer_len: usize,
    ) -> io::Result<()> {
        let num_chunks = buffer_len.div_ceil(chunk_size);

        if self.compressors.len() < num_chunks {
            self.compressors
                .reserve(num_chunks - self.compressors.len());
            while self.compressors.len() < num_chunks {
                self.compressors.push(Compressor::new(self.level));
            }
        }

        if self.output_buffers.len() < num_chunks {
            self.output_buffers
                .reserve(num_chunks - self.output_buffers.len());
            let bound = Compressor::deflate_compress_bound(chunk_size) + 5;
            while self.output_buffers.len() < num_chunks {
                self.output_buffers.push(Vec::with_capacity(bound));
            }
        }

        self.buffer
            .par_chunks(chunk_size)
            .zip(self.compressors.par_iter_mut())
            .zip(self.output_buffers.par_iter_mut())
            .enumerate()
            .try_for_each(|(i, ((chunk, compressor), output))| -> io::Result<()> {
                let mut bound = Compressor::deflate_compress_bound(chunk.len());
                if !(final_block && i == num_chunks - 1) {
                    bound += 5;
                }
                output.clear();
                if output.capacity() < bound {
                    output.try_reserve(bound).map_err(io::Error::other)?;
                }

                let mode = if final_block && i == num_chunks - 1 {
                    crate::compress::FlushMode::Finish
                } else {
                    crate::compress::FlushMode::Sync
                };
                let out_uninit = &mut output.spare_capacity_mut()[..bound];
                let (res, size, _) = compressor.compress(chunk, out_uninit, mode);
                if res == CompressResult::Success {
                    assert!(size <= bound);
                    // SAFETY: compression succeeded and initialized the first `size` bytes.
                    unsafe {
                        output.set_len(size);
                    }
                    Ok(())
                } else {
                    Err(io::Error::other("Compression failed"))
                }
            })?;

        if let Some(writer) = &mut self.writer {
            for i in 0..num_chunks {
                writer.write_all(&self.output_buffers[i])?;
            }
        }
        Ok(())
    }

    fn flush_buffer_stored(&mut self, final_block: bool) -> io::Result<()> {
        if let Some(writer) = &mut self.writer {
            let num_blocks = self.buffer.len().div_ceil(u16::MAX as usize).max(1);
            for i in 0..num_blocks {
                let start = i * u16::MAX as usize;
                let end = min(start + u16::MAX as usize, self.buffer.len());
                let len = (end - start) as u16;
                let [lo, hi] = len.to_le_bytes();
                let header = [(final_block && i + 1 == num_blocks) as u8, lo, hi, !lo, !hi];
                writer.write_all(&header)?;
                writer.write_all(&self.buffer[start..end])?;
            }
        }
        Ok(())
    }

    fn flush_buffer_sequential(&mut self, final_block: bool) -> io::Result<()> {
        if self.compressors.is_empty() {
            self.compressors.push(Compressor::new(self.level));
        }
        if self.output_buffers.is_empty() {
            let bound = Compressor::deflate_compress_bound(self.buffer.len()) + 5;
            self.output_buffers.push(Vec::with_capacity(bound));
        }

        let compressor = &mut self.compressors[0];
        let output = &mut self.output_buffers[0];
        let mut bound = Compressor::deflate_compress_bound(self.buffer.len());
        if !final_block {
            bound += 5;
        }
        output.clear();
        if output.capacity() < bound {
            output.try_reserve(bound).map_err(io::Error::other)?;
        }

        let mode = if final_block {
            crate::compress::FlushMode::Finish
        } else {
            crate::compress::FlushMode::Sync
        };
        let out_uninit = &mut output.spare_capacity_mut()[..bound];
        let (res, size, _) = compressor.compress(&self.buffer, out_uninit, mode);
        if res == CompressResult::Success {
            assert!(size <= bound);
            // SAFETY: compression succeeded and initialized the first `size` bytes.
            unsafe {
                output.set_len(size);
            }
            if let Some(writer) = &mut self.writer {
                writer.write_all(&output[..size])?;
            }
        } else {
            return Err(io::Error::other("Compression failed"));
        }
        Ok(())
    }

    fn flush_buffer(&mut self, final_block: bool) -> io::Result<()> {
        if self.failed {
            return Err(io::Error::other("encoder failed during an earlier write"));
        }
        if self.buffer.is_empty() && !final_block {
            return Ok(());
        }

        let chunk_size = 256 * 1024;
        let buffer_len = self.buffer.len();

        // A partial write cannot be replayed without corrupting the stream.
        self.failed = true;
        if self.level == 0 {
            // Stored blocks can be written directly without compressors or output buffers.
            self.flush_buffer_stored(final_block)?;
        } else if buffer_len > chunk_size {
            self.flush_buffer_parallel(final_block, chunk_size, buffer_len)?;
        } else {
            self.flush_buffer_sequential(final_block)?;
        }

        self.buffer.clear();
        self.failed = false;
        Ok(())
    }

    /// Flushes the internal buffer, finishes the compression stream, and returns the underlying writer.
    ///
    /// This method must be called to complete the compression process and handle any potential I/O errors.
    /// If this method is not called, the `Drop` implementation will attempt to finish the stream,
    /// but will silently ignore any errors.
    pub fn finish(mut self) -> io::Result<W> {
        self.flush_buffer(true)?;
        self.writer
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "encoder already finished"))
    }
}

impl<W: Write + Send> Write for DeflateEncoder<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.failed {
            return Err(io::Error::other("encoder failed during an earlier write"));
        }
        if buf.is_empty() {
            return Ok(0);
        }
        if self.buffer.len() >= self.buffer_size {
            self.flush_buffer(false)?;
        }
        let count = buf.len().min(self.buffer_size - self.buffer.len());
        self.buffer.extend_from_slice(&buf[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buffer(false)?;
        if let Some(writer) = &mut self.writer {
            writer.flush()?;
        }
        Ok(())
    }
}

impl<W: Write + Send> Drop for DeflateEncoder<W> {
    fn drop(&mut self) {
        if self.writer.is_some() && !self.failed {
            let _ = self.flush_buffer(true);
        }
    }
}

pub struct DeflateDecoder<R: Read> {
    inner: R,
    decompressor: Decompressor,
    input_buffer: Vec<u8>,
    input_pos: usize,
    input_cap: usize,
    window: Vec<u8>,
    read_pos: usize,
    write_pos: usize,
    done: bool,
}

impl<R: Read> DeflateDecoder<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            decompressor: Decompressor::new(),
            input_buffer: vec![0; 32 * 1024],
            input_pos: 0,
            input_cap: 0,
            window: vec![0; 64 * 1024],
            read_pos: 0,
            write_pos: 0,
            done: false,
        }
    }
}

impl<R: Read> Read for DeflateDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            if self.read_pos < self.write_pos {
                let count = min(buf.len(), self.write_pos - self.read_pos);
                buf[..count].copy_from_slice(&self.window[self.read_pos..self.read_pos + count]);
                self.read_pos += count;
                return Ok(count);
            }
            if self.done {
                return Ok(0);
            }

            // Preserve the DEFLATE history and make room for a full match.
            if self.window.len() - self.write_pos < crate::common::DEFLATE_MAX_MATCH_LEN {
                let shift = self.write_pos - 32 * 1024;
                self.window.copy_within(shift..self.write_pos, 0);
                self.write_pos -= shift;
                self.read_pos -= shift;
            }

            // Buffered bits may still produce output after all input bytes were consumed.
            let (res, consumed, _) = self.decompressor.decompress_streaming(
                &self.input_buffer[self.input_pos..self.input_cap],
                &mut self.window,
                &mut self.write_pos,
            );
            self.input_pos += consumed;
            if res == DecompressResult::BadData {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "deflate decompression failed",
                ));
            }
            self.done = self.decompressor.state == DecompressorState::Done;
            if self.read_pos < self.write_pos || self.done {
                continue;
            }

            if self.input_pos > 0 {
                self.input_buffer
                    .copy_within(self.input_pos..self.input_cap, 0);
                self.input_cap -= self.input_pos;
                self.input_pos = 0;
            }
            if self.input_cap == self.input_buffer.len() {
                if self.input_buffer.len() >= 1024 * 1024 {
                    return Err(io::Error::other("input buffer full"));
                }
                self.input_buffer.resize(self.input_buffer.len() * 2, 0);
            }
            let n = self.inner.read(&mut self.input_buffer[self.input_cap..])?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected EOF",
                ));
            }
            self.input_cap += n;
        }
    }
}
