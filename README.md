# libdeflate-rs

A complete Rust port of libdeflate, without C code.

## Feature

- Includes streaming processing API
- Includes batch processing API
- A highly optimized implementation, faster than C binding

## Usage

**I'm still fixing a bug that caused the performance degradation, so I can't import it for now.**

```toml
[dependencies]
libdeflate = "0.1.0"
```

## Examples

See [examples](examples)

## Environment

- Rust 1.93

## Run Benchmark

```bash
py3 gen_bench_data.py
cargo test && cargo bench
```

## License

2026 404Setup. All rights reserved. Source code is licensed under a BSD-3-Clause License.
