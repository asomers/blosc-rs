// vim: tw=80
#![allow(clippy::redundant_static_lifetimes)]
//! Rust FFI bindings for the C-Blosc block-oriented compression library
//!
//! These are raw, `unsafe` FFI bindings.  Here be dragons!  You probably
//! shouldn't use this crate directly.  Instead, you should use the
//! [`blosc`](https://crates.io/crates/blosc) crate.
include!("bindgen.rs");
