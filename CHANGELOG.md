# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased] - ReleaseDate

### Added

- Added `validate`, which requires c-blosc 1.16.0 or later
  ([#23](https://github.com/asomers/blosc-rs/pull/23))

- Implemented `(Partial)Eq` on `Clevel`, `ShuffleMode`, and `Context` and
  `(Partial)Ord` on `Clevel`.
  ([#25](https://github.com/asomers/blosc-rs/pull/25))

### Changed

- `Context::{clevel,new,shuffle,typesize}` are all const now.
  ([#16](https://github.com/asomers/blosc-rs/pull/16))

- The MSRV is now 1.60.0
  ([#22](https://github.com/asomers/blosc-rs/pull/22))

### Fixed

- All public methods now return error types that implement std::error::Error.
  ([#14](https://github.com/asomers/blosc-rs/pull/14))

### Removed
