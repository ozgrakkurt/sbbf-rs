# sbbf-rs

[![CI](https://github.com/ozgrakkurt/sbbf-rs/actions/workflows/ci.yaml/badge.svg?branch=master)](https://github.com/ozgrakkurt/sbbf-rs/actions/workflows/ci.yaml)
<a href="https://crates.io/crates/sbbf-rs">
	<img src="https://img.shields.io/crates/v/sbbf-rs.svg?style=flat-square"
	alt="Crates.io version" />
</a>

Split block bloom filter implementation.

Implementation of [parquet bloom filter spec](https://github.com/apache/parquet-format/blob/master/BloomFilter.md).

## Features
- Full runtime detection of cpu features, don't need to do `target-cpu=native` or manually turn on avx
- All stable rust
- Outputs same byte buffers on different systems. Completely cross-platform.
- no_std support
- relatively simple and low amount of code

## Caveats
- Only `unsafe` api. Safe API can be found at [sbbf-rs-safe](https://github.com/ozgrakkurt/sbbf-rs-safe).
- Dynamic dispatch to methods. (Not sure if this will effect performance so much)
- On `WASM`, unlike other targets, need to do `RUSTFLAGS="-C target-feature=+simd128"` and use nightly if you want to enable SIMD accelerated version
of filter. If user compiles without enabling `simd128`, they don't need to use nightly and the fallback implementation of a filter
will be used.
