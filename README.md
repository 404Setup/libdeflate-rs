# libdeflate-rs

A complete Rust port of libdeflate, without C code.

It's entirely powered by Gemini 3 Pro (the main model) and Gemini 3 Flash, but I had them create extensive unit and
benchmark tests to ensure accuracy. All features of libdeflate-rs are automatically enabled based on the
compile-time environment, requiring no manual configuration.

## Feature

- Includes streaming processing API
- Includes batch processing API
- A highly optimized implementation, faster than C binding

## Usage
```toml
[dependencies]
libdeflate = "[VERSION]"
```

## Examples

See [examples](examples)

## Environment

- Rust 1.92

## Run Benchmark

```bash
py3 gen_bench_data.py
cargo test && cargo bench
```

## License

2026 404Setup. All rights reserved. Source code is licensed under a BSD-3-Clause License.
