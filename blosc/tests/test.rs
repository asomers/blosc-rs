// vim: tw=80

use blosc::*;
use rand::distributions::{Distribution, Uniform};
use rstest::rstest;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem,
};

#[test]
fn test_invalid_compressor() {
    assert!(Context::new().compressor(Compressor::Invalid).is_err())
}

#[rstest]
#[case::baseline(None, None, Clevel::L2, Compressor::LZ4, ShuffleMode::Byte)]
#[case::forced_blocksize(Some(65536), None, Clevel::L2, Compressor::LZ4, ShuffleMode::Byte)]
#[case::clevel_none(None, None, Clevel::None, Compressor::LZ4, ShuffleMode::Byte)]
#[case::clevel_9(None, None, Clevel::L9, Compressor::LZ4, ShuffleMode::Byte)]
#[case::blosclz(None, None, Clevel::L2, Compressor::BloscLZ, ShuffleMode::Byte)]
#[case::lz4hc(None, None, Clevel::L2, Compressor::LZ4HC, ShuffleMode::Byte)]
#[case::snappy(None, None, Clevel::L2, Compressor::Snappy, ShuffleMode::Byte)]
#[case::zlib(None, None, Clevel::L2, Compressor::Zlib, ShuffleMode::Byte)]
#[case::zstd(None, None, Clevel::L2, Compressor::Zstd, ShuffleMode::Byte)]
#[case::nosuffle(None, None, Clevel::L2, Compressor::LZ4, ShuffleMode::None)]
#[case::bitshuffle(None, None, Clevel::L2, Compressor::LZ4, ShuffleMode::Bit)]
#[case::maxcompress(None, None, Clevel::L9, Compressor::Zstd, ShuffleMode::Bit)]
#[case::forced_typesize(None, Some(2), Clevel::L2, Compressor::LZ4, ShuffleMode::Byte)]
fn round_trip(
    #[case] blocksize: Option<usize>,
    #[case] typesize: Option<usize>,
    #[case] clevel: Clevel,
    #[case] compressor: Compressor,
    #[case] shuffle_mode: ShuffleMode,
) {
    let ctx = Context::new()
        .blocksize(blocksize)
        .clevel(clevel)
        .compressor(compressor)
        .unwrap()
        .shuffle(shuffle_mode)
        .typesize(typesize);
    let distribution = Uniform::new(1000u32, 2000u32);
    let mut rng = rand::thread_rng();
    let sample = (0..10)
        .map(|_| distribution.sample(&mut rng))
        .collect::<Vec<_>>();

    let encoded = ctx.compress(&sample[..]);
    let srclen = sample.len() * mem::size_of::<u32>();
    let ratio = srclen as f64 / encoded.size() as f64;
    println!("Compression ratio: {}", ratio);
    let decoded = decompress(&encoded).unwrap();
    assert_eq!(sample, decoded);
}

#[test]
fn test_buffer_hash() {
    let data: Vec<u8> = vec![1, 2, 3];
    let ctx = Context::new();
    let compressed = ctx.compress(&data[..]);
    let mut buffer_hasher = DefaultHasher::new();
    compressed.hash(&mut buffer_hasher);
    let mut slice_hasher = DefaultHasher::new();
    slice_hasher.write(compressed.as_ref());
    assert_eq!(buffer_hasher.finish(), slice_hasher.finish());
}

#[cfg(test)]
mod validate {
    use super::*;

    #[test]
    fn ok() {
        let data: Vec<u16> = vec![1, 2, 3, 65535];
        let ctx = Context::new();
        let compressed = ctx.compress(&data[..]);
        assert_eq!(Ok(8), validate(compressed.as_ref()));
    }

    #[test]
    fn err() {
        let compressed = vec![0u8; 8];
        validate(compressed.as_ref()).unwrap_err();
    }
}
