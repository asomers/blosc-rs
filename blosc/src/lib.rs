// vim: tw=80
//! Rust bindings for the C-BLOSC block-oriented compression library.
//!
//! Blosc is a high performance compressor optimized for binary data.  It is
//! especially good at compressing arrays of similar data.  For example, floats
//! that fit a particular statistical distribution, integers from a restricted
//! range, or pointers that all share the same alignment.  It also works well on
//! arrays of `Struct`s with similar content.
//!
//! Unlike most other compression libraries, Blosc is block-oriented, rather
//! than stream-oriented.  This works well when the entire dataset to be
//! compressed/decompressed is available at once.
//!
//! # Example
//! ```
//! # use blosc::*;
//! let data: Vec<u32> = vec![1, 1, 2, 5, 8, 13, 21, 34, 55, 89, 144];
//! let ctx = Context::new(None, Clevel::L2, Compressor::BloscLZ,
//!                        ShuffleMode::Byte).unwrap();
//! let compressed = ctx.compress(&data[..]);
//! let decompressed = unsafe { decompress(&compressed[..]) }.unwrap();
//! assert_eq!(data, decompressed);
//! ```

extern crate blosc_sys;
extern crate libc;

use blosc_sys::*;
use std::{mem, ptr};
use std::convert::Into;
use std::os::raw::{c_char, c_int, c_void};

/// The desired compression level.  Higher levels mean more compression.
#[derive(Clone, Copy, Debug)]
pub enum Clevel {
    /// No compression at all.
    ///
    /// Probably useless in single-threaded mode.
    None = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
    L6 = 6,
    L7 = 7,
    L8 = 8,
    L9 = 9
}

const BLOSC_INVALID_COMPNAME: &'static [u8; 8usize] = b"invalid\0";

/// Compressor selection.
///
/// Under the hood, Blosc supports several different compression algorithms.
#[derive(Clone, Copy, Debug)]
pub enum Compressor {
    /// The default compressor, based on FastLZ.  It's very fast, but the
    /// compression isn't as good as the other compressors.
    BloscLZ,
    /// Another fast compressor.  http://www.lz4.org/
    LZ4,
    /// Slower, higher compression version of LZ4.  http://www.lz4.org/
    LZ4HC,
    /// Another fast compressor from Google.  https://github.com/google/snappy
    Snappy,
    /// The venerable Zlib.  Slower, but better compression than most other
    /// algorithms.  https://www.zlib.net
    Zlib,
    /// A high compression algorithm from Facebook.
    /// https://facebook.github.io/zstd/
    Zstd,
    /// For testing purposes only
    #[doc(hidden)]
    Invalid
}

impl Into<*const c_char> for Compressor {
    fn into(self) -> *const c_char {
        let compref = match self {
            Compressor::BloscLZ => BLOSC_BLOSCLZ_COMPNAME.as_ptr(),
            Compressor::LZ4 => BLOSC_LZ4_COMPNAME.as_ptr(),
            Compressor::LZ4HC => BLOSC_LZ4HC_COMPNAME.as_ptr(),
            Compressor::Snappy => BLOSC_SNAPPY_COMPNAME.as_ptr(),
            Compressor::Zlib => BLOSC_ZLIB_COMPNAME.as_ptr(),
            Compressor::Zstd => BLOSC_ZSTD_COMPNAME.as_ptr(),
            Compressor::Invalid => BLOSC_INVALID_COMPNAME.as_ptr()
        };
        compref as *const c_char
    }
}

/// Controls Blosc's shuffle operation.
///
/// The Shuffle operation is the key to efficiently compressing arrays.  It
/// rearranges the array to put every entry's MSB together and every entry's LSB
/// together, which improves the performance of every Compressor.
#[derive(Clone, Copy, Debug)]
pub enum ShuffleMode {
    /// No shuffle.  Use this mode for data that is not an array.
    None = BLOSC_NOSHUFFLE as isize,

    /// Byte shuffle.  Use this mode for most arrays.
    ///
    /// https://speakerdeck.com/francescalted/new-trends-in-storing-large-data-silos-in-python
    Byte = BLOSC_SHUFFLE as isize,

    /// Bit shuffle.
    ///
    /// This is similar to the `Byte` shuffle, but works bit-by-bit instead of
    /// byte-by-byte.  It gives better compression for most datasets, but runs a
    /// little slower.  Use it when compressing numeric data if higher
    /// compression is desired.
    ///
    /// http://blosc.org/posts/new-bitshuffle-filter/
    Bit = BLOSC_BITSHUFFLE as isize,
}

/// Holds basic settings for repeated compress operations.
#[derive(Clone, Copy, Debug)]
pub struct Context {
    blocksize: Option<usize>,
    clevel: Clevel,
    compressor: Compressor,
    shuffle_mode: ShuffleMode
}

impl Context {
    /// Compress an array and return a newly allocated compressed buffer.
    pub fn compress<T>(&self, src: &[T]) -> Vec<u8> {
        let typesize = mem::size_of::<T>();
        let src_size = src.len() * typesize;
        let dest_size = src_size + BLOSC_MAX_OVERHEAD as usize;
        let mut dest: Vec<u8> = Vec::with_capacity(dest_size);
        let rsize = unsafe {
            blosc_compress_ctx(
                self.clevel as c_int,
                self.shuffle_mode as c_int,
                typesize,
                src_size,
                src.as_ptr() as *const c_void,
                dest.as_mut_ptr() as *mut c_void,
                dest_size,
                self.compressor.into(),
                self.blocksize.unwrap_or(0),
                1)
        };
        // BLOSC's docs claim that blosc_compress_ctx should never return an
        // error
        assert!(rsize >= 0,
                "C-BLOSC internal error with Context={:?}, typesize={:?} nbytes={:?} and destsize={:?}",
                self, typesize, src_size, dest_size);
        unsafe {
            dest.set_len(rsize as usize);
        }
        dest.shrink_to_fit();
        dest
    }

    /// Create a context.
    ///
    /// Returns a new context unless the `compressor` is not enabled in this
    /// build of C-BLOSC.
    ///
    /// # Parameters
    ///
    /// - `blocksize`:      Blocksize is the amount of data the compressor will
    ///                     work on at one time.  Limiting it can improve the
    ///                     CPU's cache hit rate.  Generally this should be
    ///                     `None`, in which case BLOSC will choose a sensible
    ///                     value.
    /// - `clevel`:         Compression level.  Higher values will give better
    ///                     compression at the expense of speed.
    /// - `compressor`:     Compressor algorithm to use
    /// - `shuffle_mode`:   Selects which Shuffle filter to apply before
    ///                     compression.
    pub fn new(blocksize: Option<usize>, clevel: Clevel, compressor: Compressor,
           shuffle_mode: ShuffleMode) -> Result<Self, ()> {

        let comp_ptr: *const c_char = compressor.into();
        let mut complib: *mut c_char = ptr::null_mut();
        let mut version: *mut c_char = ptr::null_mut();
        let support = unsafe {
            blosc_get_complib_info(comp_ptr,
                                   &mut complib as *mut *mut c_char,
                                   &mut version as *mut *mut c_char)
        };
        unsafe {
            libc::free(complib as *mut libc::c_void);
            libc::free(version as *mut libc::c_void);
        }
        if support >= 0 {
            Ok(Context {blocksize, clevel, compressor, shuffle_mode})
        } else {
            // Compressor not supported
            Err(())
        }
    }
}

/// Decompress the provided buffer into a newly allocated `Vec`
///
/// # Safety
///
/// This function is `unsafe` because it transmutes the decompressed data into
/// an arbitrary type.  That can cause memory errors if the type parameter
/// contains references, pointers, or does anything interesting on `Drop`.  To
/// use safely, the caller must always use `decompress` with the same type as
/// the `Context::compress` call that create it.
///
/// This function is also unsafe if the compressed buffer is untrusted.  See
/// https://github.com/Blosc/c-blosc/issues/229 .
pub unsafe fn decompress<T>(src: &[u8]) -> Result<Vec<T>, ()> {
    let typesize = mem::size_of::<T>();
    let mut nbytes: usize = 0;
    let mut _cbytes: usize = 0;
    let mut _blocksize: usize = 0;
    // Unsafe if src comes from an untrusted source.
    blosc_cbuffer_sizes(
        src.as_ptr() as *const c_void,
        &mut nbytes as *mut usize,
        &mut _cbytes as *mut usize,
        &mut _blocksize as *mut usize);
    let dest_size = nbytes / typesize;
    let mut dest: Vec<T> = Vec::with_capacity(dest_size);
    // Unsafe if src comes from an untrusted source.
    let rsize = blosc_decompress_ctx(
        src.as_ptr() as *const c_void,
        dest.as_mut_ptr() as *mut c_void,
        nbytes,
        1);
    if rsize > 0 {
        // Unsafe if T contains references or pointers
        dest.set_len(rsize as usize / typesize);
        dest.shrink_to_fit();
        Ok(dest)
    } else {
        // Buffer too small, data corrupted, decompressor not available, etc
        Err(())
    }
}
