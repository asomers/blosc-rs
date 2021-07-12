// vim: tw=80

use blosc::*;
use galvanic_test::{fixture, test_suite};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem,
};

#[test]
fn test_invalid_compressor() {
    assert!(Context::new().compressor(Compressor::Invalid).is_err())
}

test_suite! {
    name round_trip;

    use super::*;
    use rand::distributions::{Distribution, Uniform};

    fixture!(settings(blocksize: Option<usize>, typesize: Option<usize>,
                      clevel: Clevel,
                      compressor: Compressor,
                      shuffle_mode: ShuffleMode) -> Context {
        params {
            vec![
                // Baseline
                (None, None, Clevel::L2, Compressor::LZ4, ShuffleMode::Byte),
                // Forced blocksize
                (Some(65536), None, Clevel::L2, Compressor::LZ4, ShuffleMode::Byte),
                // Various compression levels
                (None, None, Clevel::None, Compressor::LZ4, ShuffleMode::Byte),
                (None, None, Clevel::L9, Compressor::LZ4, ShuffleMode::Byte),
                // All different compressors
                (None, None, Clevel::L2, Compressor::BloscLZ, ShuffleMode::Byte),
                (None, None, Clevel::L2, Compressor::LZ4HC, ShuffleMode::Byte),
                (None, None, Clevel::L2, Compressor::Snappy, ShuffleMode::Byte),
                (None, None, Clevel::L2, Compressor::Zlib, ShuffleMode::Byte),
                (None, None, Clevel::L2, Compressor::Zstd, ShuffleMode::Byte),
                // Shuffle options
                (None, None, Clevel::L2, Compressor::LZ4, ShuffleMode::None),
                (None, None, Clevel::L2, Compressor::LZ4, ShuffleMode::Bit),
                // Maximum compression
                (None, None, Clevel::L9, Compressor::Zstd, ShuffleMode::Bit),
                // Forced typesize
                (None, Some(2), Clevel::L2, Compressor::LZ4, ShuffleMode::Byte),
            ].into_iter()
        }

        setup(&mut self) {
            Context::new()
                .blocksize(*self.blocksize)
                .clevel(*self.clevel)
                .compressor(*self.compressor).unwrap()
                .shuffle(*self.shuffle_mode)
                .typesize(*self.typesize)
        }
    });

    test round_trip(settings) {
        let distribution = Uniform::new(1000u32, 2000u32);
        let mut rng = rand::thread_rng();
        let sample = (0..10).map(|_| {
            distribution.sample(&mut rng)
        }).collect::<Vec<_>>();


        let encoded = settings.val.compress(&sample[..]);
        let srclen = sample.len() * mem::size_of::<u32>();
        let ratio = srclen as f64 / encoded.size() as f64;
        println!("Compression ratio: {}", ratio);
        let decoded = decompress(&encoded).unwrap();
        assert_eq!(sample, decoded);
    }
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
