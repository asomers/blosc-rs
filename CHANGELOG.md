# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased] - ReleaseDate
### Added
### Changed

- `Context::{clevel,new,shuffle,typesize}` are all const now.
  ([#16](https://github.com/nix-rust/nix/pull/16))

### Fixed

- All public methods now return error types that implement std::error::Error.
  ([#14](https://github.com/nix-rust/nix/pull/14))

### Removed
