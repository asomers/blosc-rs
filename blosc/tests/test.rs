// vim: tw=80

extern crate blosc;
#[macro_use] extern crate galvanic_test;
extern crate rand;

use blosc::*;
use std::mem;

// Ignored due to https://github.com/Blosc/c-blosc/issues/227 .
#[ignore]
#[test]
fn test_invalid_compressor() {
    assert!(Context::new().compressor(Compressor::Invalid).is_err())
}

test_suite! {
    name round_trip;

    use super::*;
    use rand::distributions::{Range, IndependentSample};

    fixture!(settings(blocksize: Option<usize>, clevel: Clevel,
                      compressor: Compressor,
                      shuffle_mode: ShuffleMode) -> Context {
        params {
            vec![
                // Baseline
                (None, Clevel::L2, Compressor::LZ4, ShuffleMode::Byte),
                // Forced blocksize
                (Some(65536), Clevel::L2, Compressor::LZ4, ShuffleMode::Byte),
                // Various compression levels
                (None, Clevel::None, Compressor::LZ4, ShuffleMode::Byte),
                (None, Clevel::L9, Compressor::LZ4, ShuffleMode::Byte),
                // All different compressors
                (None, Clevel::L2, Compressor::BloscLZ, ShuffleMode::Byte),
                (None, Clevel::L2, Compressor::LZ4HC, ShuffleMode::Byte),
                (None, Clevel::L2, Compressor::Snappy, ShuffleMode::Byte),
                (None, Clevel::L2, Compressor::Zlib, ShuffleMode::Byte),
                (None, Clevel::L2, Compressor::Zstd, ShuffleMode::Byte),
                // Shuffle options
                (None, Clevel::L2, Compressor::LZ4, ShuffleMode::None),
                (None, Clevel::L2, Compressor::LZ4, ShuffleMode::Bit),
                // Maximum compression
                (None, Clevel::L9, Compressor::Zstd, ShuffleMode::Bit),
            ].into_iter()
        }

        setup(&mut self) {
            Context::new()
                .blocksize(*self.blocksize)
                .clevel(*self.clevel)
                .compressor(*self.compressor).unwrap()
                .shuffle(*self.shuffle_mode)
        }
    });

    test round_trip(settings) {
        let distribution = Range::new(1000u32, 2000u32);
        let mut rng = rand::thread_rng();
        let sample = (0..1000).map(|_| {
            distribution.ind_sample(&mut rng)
        }).collect::<Vec<_>>();


        let encoded = settings.val.compress(&sample[..]);
        let srclen = sample.len() * mem::size_of::<u32>();
        let ratio = srclen as f64 / encoded.size() as f64;
        println!("Compression ratio: {}", ratio);
        let decoded = decompress(&encoded).unwrap();
        assert_eq!(sample, decoded);
    }
}
