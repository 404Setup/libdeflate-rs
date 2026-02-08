import sys

filepath = "src/compress/mod.rs"

with open(filepath, "r") as f:
    content = f.read()

target = """             let compressed_chunks_res: Vec<io::Result<Vec<u8>>> = chunks.par_iter().enumerate().map_init(
                  || (Compressor::new(self.compression_level), Vec::with_capacity(chunk_size + chunk_size / 2)),
                  |(compressor, buf), (i, chunk)| {
                       let is_last = i == chunks.len() - 1;
                       let mode = if is_last { flush_mode } else { FlushMode::Sync };

                       let bound = Self::deflate_compress_bound(chunk.len());
                       if buf.capacity() < bound {
                           buf.reserve(bound - buf.len());
                       }
                       unsafe { buf.set_len(bound); }

                       let (res, size, _) = compressor.compress(chunk, buf, mode);
                       if res == CompressResult::Success {
                           unsafe { buf.set_len(size); }
                           Ok(buf.clone())
                       } else {
                           Err(io::Error::new(io::ErrorKind::Other, "Compression failed"))
                       }
                  }
             ).collect();"""

replacement = """             let compressed_chunks_res: Vec<io::Result<Vec<u8>>> = chunks.par_iter().enumerate().map_init(
                  || Compressor::new(self.compression_level),
                  |compressor, (i, chunk)| {
                       let is_last = i == chunks.len() - 1;
                       let mode = if is_last { flush_mode } else { FlushMode::Sync };

                       let bound = Self::deflate_compress_bound(chunk.len());
                       let mut buf = Vec::with_capacity(bound);
                       unsafe { buf.set_len(bound); }

                       let (res, size, _) = compressor.compress(chunk, &mut buf, mode);
                       if res == CompressResult::Success {
                           unsafe { buf.set_len(size); }
                           Ok(buf)
                       } else {
                           Err(io::Error::new(io::ErrorKind::Other, "Compression failed"))
                       }
                  }
             ).collect();"""

if target in content:
    new_content = content.replace(target, replacement)
    with open(filepath, "w") as f:
        f.write(new_content)
    print("Successfully replaced code.")
else:
    print("Target code not found!")
    # Debug: print near match or context
    start_marker = "let compressed_chunks_res"
    idx = content.find(start_marker)
    if idx != -1:
        print("Found start marker. Context:")
        print(content[idx:idx+500])
    else:
        print("Start marker not found.")
    sys.exit(1)
