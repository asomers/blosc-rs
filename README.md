# Blosc-rs

Rust bindings for the C-Blosc compression library.

[![Build Status](https://api.cirrus-ci.com/github/asomers/blosc-rs.svg?maxAge=2592000)](https://cirrus-ci.com/github/asomers/blosc-rs)
[![Crates.io](https://img.shields.io/crates/v/blosc.svg?maxAge=2592000)](https://crates.io/crates/blosc)
[![Crates.io](https://img.shields.io/crates/v/blosc-sys.svg?maxAge=2592000)](https://crates.io/crates/blosc-sys)

The `blosc` crate provides Rusty bindings for [`C-Blosc`](http://blosc.org/), a
compression library for binary data, especially numeric arrays.  The
`blosc-sys` crate provides raw FFI bindings for C-Blosc.  You probably don't
want to use it directly.

# Usage

```toml
# Cargo.toml
[dependencies]
blosc = "0.2"
```

```rust
extern crate blosc;

fn main() {
    let data: Vec<u32> = vec![1, 1, 2, 5, 8, 13, 21, 34, 55, 89, 144];
    let ctx = blosc::Context::new();
    let compressed = ctx.compress(&data[..]);
    let decompressed = decompress(&compressed).unwrap();
    assert_eq!(data, decompressed);
}
```

# License
`blosc` and `blosc-sys` are distributed under the MIT license.  See
[LICENSE-MIT](blosc/LICENSE-MIT) for details.
