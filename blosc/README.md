# Blosc

Rust bindings for the C-Blosc compression library.

[![Build Status](https://travis-ci.org/asomers/blosc-rs.svg?branch=master)](https://travis-ci.org/asomers/blosc-rs)

The `blosc` crate provides Rusty bindings for [`C-Blosc`](http://blosc.org/), a
compression library for binary data, especially numeric arrays.

# Usage

```toml
# Cargo.toml
[dependencies]
blosc = "0.1"
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
`blosc` is distributed under the MIT license.  See
[LICENSE-MIT](blosc/LICENSE-MIT) for details.
