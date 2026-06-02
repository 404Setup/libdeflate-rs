use libdeflate::stream::DeflateEncoder;
use std::io::Write;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct SharedWriter {
    data: Arc<Mutex<Vec<u8>>>,
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut data = self.data.lock().unwrap();
        data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_with_buffer_size() {
    let writer_data = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedWriter {
        data: writer_data.clone(),
    };

    let buffer_size = 100;
    // Buffer size 100, we write 150 bytes.
    let mut encoder = DeflateEncoder::new(writer, 1).with_buffer_size(buffer_size);

    let data = vec![0u8; 150];
    encoder.write_all(&data).unwrap();

    // The buffer size (100) is exceeded by 150 bytes, so flush_buffer(false) should be called.
    // flush_buffer compresses and writes to the underlying writer.
    let compressed_len = writer_data.lock().unwrap().len();
    assert!(
        compressed_len > 0,
        "Encoder should have flushed when buffer limit was exceeded"
    );

    // Finish the stream
    encoder.finish().unwrap();

    let final_len = writer_data.lock().unwrap().len();
    assert!(
        final_len > compressed_len,
        "Finish should write more data (footer/final block)"
    );
}

#[test]
fn test_default_buffer_size() {
    let writer_data = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedWriter {
        data: writer_data.clone(),
    };

    // Default buffer size is usually large (e.g. 1MB).
    let mut encoder = DeflateEncoder::new(writer, 1);

    let data = vec![0u8; 150];
    encoder.write_all(&data).unwrap();

    // Should not have flushed yet as 150 < default buffer size
    let compressed_len = writer_data.lock().unwrap().len();
    assert_eq!(
        compressed_len, 0,
        "Encoder should NOT have flushed with default buffer size"
    );

    encoder.finish().unwrap();
    assert!(writer_data.lock().unwrap().len() > 0);
}

#[test]
fn test_small_vs_large_buffer() {
    use libdeflate::stream::DeflateDecoder;
    use std::io::{Cursor, Read};

    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

    let writer_small = Arc::new(Mutex::new(Vec::new()));
    let writer_large = Arc::new(Mutex::new(Vec::new()));

    let mut encoder_small = DeflateEncoder::new(SharedWriter { data: writer_small.clone() }, 6).with_buffer_size(10);
    let mut encoder_large = DeflateEncoder::new(SharedWriter { data: writer_large.clone() }, 6).with_buffer_size(10000);

    encoder_small.write_all(&data).unwrap();
    encoder_small.finish().unwrap();

    encoder_large.write_all(&data).unwrap();
    encoder_large.finish().unwrap();

    let small_compressed = writer_small.lock().unwrap().clone();
    let large_compressed = writer_large.lock().unwrap().clone();

    // Verify small buffer compresses data correctly
    let mut decoder = DeflateDecoder::new(Cursor::new(&small_compressed));
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    assert_eq!(data, decompressed);

    // Verify large buffer compresses data correctly
    let mut decoder_large = DeflateDecoder::new(Cursor::new(&large_compressed));
    let mut decompressed_large = Vec::new();
    decoder_large.read_to_end(&mut decompressed_large).unwrap();
    assert_eq!(data, decompressed_large);

    // In DEFLATE, different flush boundaries might technically produce slightly different output sizes
    // depending on the compression level and exact boundary, but they both should successfully decompress
    // to the same original data, and the sizes shouldn't be radically different.
    // The exact byte-for-byte comparison of small_compressed and large_compressed might fail
    // if flushing changes blocks, but we can verify both yield valid, equivalent uncompressed data
    // and that the small buffer works correctly over multiple chunks.
    assert!(small_compressed.len() > 0);
    assert!(large_compressed.len() > 0);
}
