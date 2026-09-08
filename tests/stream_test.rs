use libdeflate::stream::{DeflateDecoder, DeflateEncoder};
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};

struct ChunkedReader<R>(R, usize);

impl<R: Read> Read for ChunkedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        assert!(!buf.is_empty(), "empty read must not access the source");
        let size = buf.len().min(self.1);
        self.0.read(&mut buf[..size])
    }
}

#[derive(Clone)]
struct FlushTrackingWriter {
    data: Arc<Mutex<Vec<u8>>>,
    flush_count: Arc<Mutex<usize>>,
}

impl Write for FlushTrackingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        *self.flush_count.lock().unwrap() += 1;
        Ok(())
    }
}

struct ErrorFlushWriter;

impl Write for ErrorFlushWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "flush error",
        ))
    }
}

#[test]
fn test_stream_round_trip() {
    let mut data = Vec::with_capacity(10000);
    for i in 0..10000 {
        data.push((i % 256) as u8);
    }

    let mut encoder = DeflateEncoder::new(Vec::new(), 6);
    encoder.write_all(&data).unwrap();
    let compressed = encoder.finish().unwrap();

    let mut decoder = DeflateDecoder::new(Cursor::new(compressed));
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();

    assert_eq!(data, decompressed);
}

#[test]
fn test_stream_small_chunks() {
    let mut data = Vec::with_capacity(10000);
    for i in 0..10000 {
        data.push((i % 256) as u8);
    }

    let mut encoder = DeflateEncoder::new(Vec::new(), 6);
    encoder.write_all(&data).unwrap();
    let compressed = encoder.finish().unwrap();

    let mut decoder = DeflateDecoder::new(Cursor::new(compressed));
    let mut decompressed = Vec::new();
    let mut buf = [0u8; 10];
    loop {
        let n = decoder.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        decompressed.extend_from_slice(&buf[..n]);
    }

    assert_eq!(data, decompressed);
}

#[test]
fn test_encoder_flush() {
    let data = Arc::new(Mutex::new(Vec::new()));
    let flush_count = Arc::new(Mutex::new(0));
    let writer = FlushTrackingWriter {
        data: data.clone(),
        flush_count: flush_count.clone(),
    };

    let mut encoder = DeflateEncoder::new(writer, 6);
    encoder.write_all(b"Hello World").unwrap();
    encoder.flush().unwrap();

    // Verify data was written (compressed)
    assert!(!data.lock().unwrap().is_empty());

    // Verify flush was called on the underlying writer
    assert_eq!(*flush_count.lock().unwrap(), 1);
}

#[test]
fn test_encoder_flush_error() {
    let writer = ErrorFlushWriter;
    let mut encoder = DeflateEncoder::new(writer, 6);
    encoder.write_all(b"Hello World").unwrap();

    // flush() should fail because the underlying writer returns an error
    assert!(encoder.flush().is_err());
}

#[test]
fn test_decoder_window_and_fragmented_input() {
    let mut seed = 12345u32;
    let history: Vec<u8> = (0..32768)
        .map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        })
        .collect();
    for data in [
        (0..200_000).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
        history.repeat(6),
    ] {
        for level in [0, 1, 6] {
            let mut compressor =
                libdeflater::Compressor::new(libdeflater::CompressionLvl::new(level).unwrap());
            let mut compressed = vec![0; compressor.deflate_compress_bound(data.len())];
            let size = compressor.deflate_compress(&data, &mut compressed).unwrap();
            compressed.truncate(size);
            for chunk_size in [1, 7, 32768] {
                let reader = ChunkedReader(compressed.as_slice(), chunk_size);
                let mut decoder = DeflateDecoder::new(reader);
                let mut output = Vec::new();
                decoder.read_to_end(&mut output).unwrap();
                assert_eq!(output.len(), data.len(), "input chunk size {chunk_size}");
                assert_eq!(output, data);
            }
        }
    }
}

#[test]
fn test_decoder_empty_read_does_not_read_input() {
    let reader = ChunkedReader(&[7][..], 1);
    assert_eq!(DeflateDecoder::new(reader).read(&mut []).unwrap(), 0);
}

#[test]
fn test_decoder_rejects_missing_final_block() {
    for input in [&[][..], &[0x03][..], &[0, 0, 0, 255, 255][..]] {
        let error = DeflateDecoder::new(input)
            .read_to_end(&mut Vec::new())
            .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    }
}

#[test]
fn test_encoder_bounds_large_writes() {
    let data = vec![42; 2 * 1024 * 1024];
    for buffer_size in [0, 1024, 1024 * 1024] {
        let mut encoder = DeflateEncoder::new(Vec::new(), 1).with_buffer_size(buffer_size);
        let count = encoder.write(&data).unwrap();
        assert_eq!(count, buffer_size.max(1));
        if buffer_size != 0 {
            encoder.write_all(&data[count..]).unwrap();
        }
        let compressed = encoder.finish().unwrap();
        let expected = if buffer_size == 0 {
            &data[..1]
        } else {
            &data[..]
        };
        let mut output = vec![0; expected.len()];
        let size = libdeflater::Decompressor::new()
            .deflate_decompress(&compressed, &mut output)
            .unwrap();
        assert_eq!(&output[..size], expected);
    }
}

#[test]
fn test_stored_encoder_boundaries_and_flush() {
    for size in [0, 1, 65535, 65536, 1024 * 1024 + 1] {
        let data: Vec<u8> = (0..size).map(|i| i as u8).collect();
        for flush in [false, true] {
            let mut encoder = DeflateEncoder::new(Vec::new(), 0);
            encoder.write_all(&data).unwrap();
            if flush {
                encoder.flush().unwrap();
                encoder.flush().unwrap();
                encoder.write_all(&data).unwrap();
            }
            let compressed = encoder.finish().unwrap();
            let expected = if flush { data.repeat(2) } else { data.clone() };
            let mut output = vec![0; expected.len()];
            assert_eq!(
                libdeflater::Decompressor::new()
                    .deflate_decompress(&compressed, &mut output)
                    .unwrap(),
                expected.len()
            );
            assert_eq!(output, expected);
        }
    }
}

#[test]
fn test_encoder_does_not_retry_partial_write_on_drop() {
    struct FailAfterPrefix(Arc<Mutex<usize>>);
    impl Write for FailAfterPrefix {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            let mut calls = self.0.lock().unwrap();
            *calls += 1;
            if *calls == 1 {
                Ok(1)
            } else {
                Err(std::io::Error::other("write failed"))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    for level in [0, 1] {
        let calls = Arc::new(Mutex::new(0));
        let mut encoder = DeflateEncoder::new(FailAfterPrefix(calls.clone()), level);
        encoder
            .write_all(b"partial writes must not be replayed")
            .unwrap();
        assert!(encoder.finish().is_err());
        assert_eq!(*calls.lock().unwrap(), 2);
    }
}

#[test]
fn test_streaming_decompressor_resumes_after_output_full() {
    use libdeflate::decompress::{DecompressResult, Decompressor};
    let data: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
    let compressed = libdeflate::Compressor::new(6)
        .unwrap()
        .compress_deflate(&data)
        .unwrap();
    for first_size in [0, 1, 250, 252, 1024] {
        let mut decoder = Decompressor::new();
        let mut output = vec![0; data.len()];
        let mut written = 0;
        let (result, consumed, _) =
            decoder.decompress_streaming(&compressed, &mut output[..first_size], &mut written);
        assert_eq!(result, DecompressResult::InsufficientSpace);
        let (result, _, _) =
            decoder.decompress_streaming(&compressed[consumed..], &mut output, &mut written);
        assert_eq!(result, DecompressResult::Success);
        assert_eq!(written, data.len());
        assert_eq!(output, data);
    }
}
