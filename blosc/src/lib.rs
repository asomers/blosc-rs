// vim: tw=80
//! Rust bindings for the C-Blosc block-oriented compression library.
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
//! let ctx = Context::new();
//! let compressed = ctx.compress(&data[..]);
//! let decompressed = decompress(&compressed).unwrap();
//! assert_eq!(data, decompressed);
//! ```

use blosc_sys::*;
use std::{
    convert::Into,
    error,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    os::raw::{c_char, c_int, c_void},
    {mem, ptr},
};

/// An unspecified error from C-Blosc
#[derive(Clone, Copy, Debug)]
pub struct BloscError;

impl fmt::Display for BloscError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unspecified error from c-Blosc")
    }
}

impl error::Error for BloscError {}

pub type Result<T> = std::result::Result<T, BloscError>;

/// The desired compression level.  Higher levels mean more compression.
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
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
    L9 = 9,
}

const BLOSC_INVALID_COMPNAME: &[u8; 8usize] = b"invalid\0";

/// Compressor selection.
///
/// Under the hood, Blosc supports several different compression algorithms.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Compressor {
    /// The default compressor, based on FastLZ.  It's very fast, but the
    /// compression isn't as good as the other compressors.
    BloscLZ,
    /// Another fast compressor.  See [lz4.org](http://www.lz4.org).
    LZ4,
    /// Slower, higher compression version of LZ4.
    /// See [lz4.org](http://www.lz4.org).
    LZ4HC,
    /// Another fast compressor from Google.  See
    /// [Snappy](https://github.com/google/snappy)
    Snappy,
    /// The venerable Zlib.  Slower, but better compression than most other
    /// algorithms.  See [zlib.net](https://www.zlib.net)
    Zlib,
    /// A high compression algorithm from Facebook.
    /// See [zstd](https://facebook.github.io/zstd).
    Zstd,
    /// For testing purposes only
    #[doc(hidden)]
    Invalid,
}

impl From<Compressor> for *const c_char {
    fn from(compressor: Compressor) -> Self {
        let compref = match compressor {
            Compressor::BloscLZ => BLOSC_BLOSCLZ_COMPNAME.as_ptr(),
            Compressor::LZ4 => BLOSC_LZ4_COMPNAME.as_ptr(),
            Compressor::LZ4HC => BLOSC_LZ4HC_COMPNAME.as_ptr(),
            Compressor::Snappy => BLOSC_SNAPPY_COMPNAME.as_ptr(),
            Compressor::Zlib => BLOSC_ZLIB_COMPNAME.as_ptr(),
            Compressor::Zstd => BLOSC_ZSTD_COMPNAME.as_ptr(),
            Compressor::Invalid => BLOSC_INVALID_COMPNAME.as_ptr(),
        };
        compref as *const c_char
    }
}

/// Controls Blosc's shuffle operation.
///
/// The Shuffle operation is the key to efficiently compressing arrays.  It
/// rearranges the array to put every entry's MSB together and every entry's LSB
/// together, which improves the performance of every `Compressor`.
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum ShuffleMode {
    /// No shuffle.  Use this mode for data that is not an array.
    None = BLOSC_NOSHUFFLE as i32,

    /// Byte shuffle.  Use this mode for most arrays.
    ///
    /// See [new-trends-in-storing-large-data-silos-in-python](https://speakerdeck.com/francescalted/new-trends-in-storing-large-data-silos-in-python)
    Byte = BLOSC_SHUFFLE as i32,

    /// Bit shuffle.
    ///
    /// This is similar to the `Byte` shuffle, but works bit-by-bit instead of
    /// byte-by-byte.  It gives better compression for most datasets, but runs a
    /// little slower.  Use it when compressing numeric data if higher
    /// compression is desired.
    ///
    /// See [new-bitshuffle-filter](http://blosc.org/posts/new-bitshuffle-filter/)
    Bit = BLOSC_BITSHUFFLE as i32,
}

/// Holds basic settings for `compress` operations.
// LCOV_EXCL_START
#[derive(Clone, Copy, Debug)]
pub struct Context {
    blocksize: usize,
    clevel: Clevel,
    compressor: Compressor,
    shuffle_mode: ShuffleMode,
    typesize: Option<usize>,
}
// LCOV_EXCL_STOP

/// An opaque Blosc-compressed buffer.
///
/// It can be safely decompressed back into an array of the original type.
pub struct Buffer<T> {
    data: Vec<u8>,
    phantom: PhantomData<T>,
}

impl<T> Buffer<T> {
    fn from_vec(src: Vec<u8>) -> Self {
        Buffer {
            data: src,
            phantom: PhantomData,
        }
    }

    /// Return the size of the compressed buffer.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl<T> AsRef<[u8]> for Buffer<T> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<T> Hash for Buffer<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write(self.as_ref());
    }
}

impl<T> From<Buffer<T>> for Vec<u8> {
    /// Transform `self` into a raw `Vec` of bytes.  After this, it can no
    /// longer be safely decompressed.
    fn from(buf: Buffer<T>) -> Self {
        buf.data
    }
}

impl Context {
    /// Select the `Context`'s blocksize.
    ///
    /// Blocksize is the amount of data the compressor will work on at one time.
    /// Limiting it can improve the CPU's cache hit rate.  Increasing it can
    /// improve compression.  Generally this should be `None`, in which case
    /// Blosc will choose a sensible value.
    pub fn blocksize(mut self, blocksize: Option<usize>) -> Self {
        self.blocksize = blocksize.unwrap_or(0);
        self
    }

    /// Select the `Context`'s compression level.
    ///
    /// Higher values will give better compression at the expense of speed.
    pub const fn clevel(mut self, clevel: Clevel) -> Self {
        self.clevel = clevel;
        self
    }

    /// Select the `Context`'s compression algorithm.
    ///
    /// Returns an error if the `compressor` is not enabled in this build of
    /// C-Blosc.
    pub fn compressor(mut self, compressor: Compressor) -> Result<Self> {
        let comp_ptr: *const c_char = compressor.into();
        let support = unsafe { blosc_get_complib_info(comp_ptr, ptr::null_mut(), ptr::null_mut()) };
        if support >= 0 {
            self.compressor = compressor;
            Ok(self)
        } else {
            // Compressor not supported
            Err(BloscError)
        }
    }

    /// Compress an array and return a newly allocated compressed buffer.
    pub fn compress<T>(&self, src: &[T]) -> Buffer<T> {
        let typesize = self.typesize.unwrap_or(mem::size_of::<T>());
        let src_size = src.len() * mem::size_of::<T>();
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
                self.blocksize,
                1,
            )
        };
        // Blosc's docs claim that blosc_compress_ctx should never return an
        // error
        // LCOV_EXCL_START
        assert!(
            rsize >= 0,
            "C-Blosc internal error with Context={:?}, typesize={:?} nbytes={:?} and destsize={:?}",
            self,
            typesize,
            src_size,
            dest_size
        );
        // LCOV_EXCL_STOP
        unsafe {
            dest.set_len(rsize as usize);
        }
        dest.shrink_to_fit();
        Buffer::from_vec(dest)
    }

    /// Build a default compression context.
    ///
    /// # Example
    ///
    /// ```
    /// # use blosc::*;
    /// # #[allow(unused)]
    /// let ctx = Context::new()
    ///     .blocksize(Some(262144))
    ///     .compressor(Compressor::Zstd).unwrap()
    ///     .clevel(Clevel::L9)
    ///     .shuffle(ShuffleMode::Bit);
    /// ```
    pub const fn new() -> Self {
        Context {
            blocksize: 0,                    // Automatic blocksize
            clevel: Clevel::L2,              // Level 2 selects blocksize to equal L1 cache
            compressor: Compressor::BloscLZ, // Default algorithm
            shuffle_mode: ShuffleMode::None, // Don't shuffle by default
            typesize: None,                  // autodetect by default
        }
    }

    /// Select which Shuffle filter to apply before compression.
    pub const fn shuffle(mut self, shuffle_mode: ShuffleMode) -> Self {
        self.shuffle_mode = shuffle_mode;
        self
    }

    /// Manually set the size in bytes to assume for each uncompressed array
    /// element.
    ///
    /// The `typesize` is used for Blosc's shuffle operation.  When compressing
    /// arrays, the `typesize` should be the size of each array element.  If
    /// `None` or unspecified, it will be autodetected.  However, manually
    /// setting `typesize` can be useful when compressing preserialized buffers
    /// or single structures that contain arrays.
    ///
    /// # Examples
    ///
    /// Set the `typesize` when compressing an array-containing struct
    ///
    /// ```
    /// # use blosc::*;
    /// # use std::mem;
    /// #[derive(Default)]
    /// struct Foo {
    ///     x: usize,
    ///     y: [u32; 32]
    /// }
    /// let foo = [Foo::default()];
    /// let ctx = Context::new().typesize(Some(mem::size_of_val(&foo[0].y[0])));
    /// ctx.compress(&foo[..]);
    /// ```
    ///
    /// Set the `typesize` when compressing preserialized data.
    ///
    /// ```
    /// # extern crate bincode;
    /// # extern crate blosc;
    /// # extern crate serde;
    /// # use blosc::*;
    /// # use std::mem;
    /// let raw: Vec<i16> = vec![0, 1, 2, 3, 4, 5];
    /// let serialized = bincode::serialize(&raw).unwrap();
    /// let ctx = Context::new().typesize(Some(mem::size_of::<i16>()));
    /// ctx.compress(&serialized[..]);
    /// ```
    pub const fn typesize(mut self, typesize: Option<usize>) -> Self {
        self.typesize = typesize;
        self
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Decompress a `blosc::Buffer` into a newly allocated `Vec`
///
/// # Safety
///
/// `decompress` is safe to use because the compiler will guarantee that `src`
/// came from the output of `Context::compress`.
///
/// # Example
/// The compiler won't allow decompressing into the wrong type.
///
/// ```compile_fail
/// # use blosc::*;
/// let data: Vec<u16> = vec![1, 2, 3, 65535];
/// let ctx = Context::new();
/// let compressed = ctx.compress(&data[..]);
/// let decompressed: Vec<i16> = decompress(&compressed).unwrap();
/// ```
pub fn decompress<T>(src: &Buffer<T>) -> Result<Vec<T>> {
    unsafe { decompress_bytes(&src.data[..]) }
}

/// Decompress arbitrary data into a newly allocated `Vec`
///
/// Use this method when decompressing serialized data from disk, or receiving
/// it over the network.
///
/// # Safety
///
/// This function is `unsafe` because it can transmute data into an arbitrary
/// type.  That can cause memory errors if the type parameter contains
/// references, pointers, or does anything interesting on `Drop`.  To use
/// safely, the caller must ensure that the serialized data really was created
/// by Blosc, with the correct type.
///
/// This function is also unsafe if the compressed buffer is untrusted.  See
/// [Blosc issue #229](https://github.com/Blosc/c-blosc/issues/229).
///
/// # Example
/// ```
/// # use blosc::*;
/// let serialized: Vec<u8> = vec![2, 1, 19, 4, 12, 0, 0, 0, 12, 0, 0, 0,
///     28, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0];
/// let decompressed = unsafe{ decompress_bytes(&serialized[..])}.unwrap();
/// assert_eq!(&[1, 2, 3], &decompressed[..]);
/// ```
pub unsafe fn decompress_bytes<T>(src: &[u8]) -> Result<Vec<T>> {
    let typesize = mem::size_of::<T>();
    let mut nbytes: usize = 0;
    let mut _cbytes: usize = 0;
    let mut _blocksize: usize = 0;
    // Unsafe if src comes from an untrusted source.
    blosc_cbuffer_sizes(
        src.as_ptr() as *const c_void,
        &mut nbytes as *mut usize,
        &mut _cbytes as *mut usize,
        &mut _blocksize as *mut usize,
    );
    let dest_size = nbytes / typesize;
    let mut dest: Vec<T> = Vec::with_capacity(dest_size);
    // Unsafe if src comes from an untrusted source.
    let rsize = blosc_decompress_ctx(
        src.as_ptr() as *const c_void,
        dest.as_mut_ptr() as *mut c_void,
        nbytes,
        1,
    );
    if rsize > 0 {
        // Unsafe if T contains references or pointers
        dest.set_len(rsize as usize / typesize);
        dest.shrink_to_fit();
        Ok(dest)
    } else {
        // Buffer too small, data corrupted, decompressor not available, etc
        Err(BloscError)
    }
}

#[test]
fn test_buffer_into() {
    let v0 = vec![0u8, 1, 2, 3, 4, 5];
    let v1 = v0.clone();
    let buf = Buffer::<u16>::from_vec(v0);
    let v2: Vec<u8> = buf.into();
    assert_eq!(v1, v2);
}
