## [Unreleased][unreleased]

### Added

* Example for displaying FLAC metadata
* Alias for `Stream<P: StreamProducer>`. `StreamReader<R: Read>` for
  `Stream<ReadStream<R: Read>>` and `StreamBuffer` for
  `Stream<ByteStream>`

### Fixed

* Infinite loop in Rust beta ([#2](https://github.com/sourrust/flac/issues/2))

## 0.1.0 - 2016-01-08

### Added

* API for dealing with metadata
* Complete parsing of FLAC files
* Complete decoding of FLAC files
* Example decoder from FLAC to WAV

[unreleased]: https://github.com/sourrust/flac/compare/v0.1.0...HEAD
