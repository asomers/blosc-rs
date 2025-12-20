# Blosc

Rust bindings for the C-Blosc compression library.

[![Build Status](https://travis-ci.org/asomers/blosc-rs.svg?branch=master)](https://travis-ci.org/asomers/blosc-rs)

The `blosc` crate provides Rusty bindings for [`C-Blosc`](http://blosc.org/), a
compression library for binary data, especially numeric arrays.

> [!WARNING]
> This crate is deprecated.  The author no longer uses it in his own projects,
> and new alternatives have arisen.  If you use this crate, then I suggest you
> either:
>
> * Volunteer to take over maintainership from me.
> * Switch to the [byteshuffle](https://crates.io/crates/byteshuffle) crate, if
>   all you care about is the shuffle filter.
> * Switch to the [blosc-rs](https://crates.io/crates/blosc-rs/) crate.
> * Switch to the [blosc2](https://crates.io/crates/blosc2) crate.
> * Switch to the [blosc2-rs](https://crates.io/crates/blosc2-rs) crate.

# Usage

```toml
# Cargo.toml
[dependencies]
blosc = "0.3"
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
