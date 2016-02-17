## [Unreleased]

### Changed

* Variant `PictureType::DuringPerformace` to
  `PictureType::DuringPerformance`, there was a missing "n"

## [0.2.0] - 2016-02-06

### Added

* Example for displaying FLAC metadata
* Alias for `Stream<P: StreamProducer>`. `StreamReader<R: Read>` for
  `Stream<ReadStream<R: Read>>` and `StreamBuffer` for
  `Stream<ByteStream>`

### Fixed

* Infinite loop in Rust beta
  ([#2](https://github.com/sourrust/flac/issues/2))
* Out of memory error on Linux
  ([#3](https://github.com/sourrust/flac/issues/3))

## 0.1.0 - 2016-01-08

### Added

* API for dealing with metadata
* Complete parsing of FLAC files
* Complete decoding of FLAC files
* Example decoder from FLAC to WAV

[Unreleased]: https://github.com/sourrust/flac/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/sourrust/flac/compare/v0.1.0...v0.2.0
