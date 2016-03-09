## [Unreleased]

### Added

* `StreamInfo` methods for checking block size range:
  - `is_varied_block_size`
  - `is_fixed_block_size`
* `Type` enum for metadata block type
* `Metadata::data_type` for returning the `Type` of the current metadata
* `Metadata` methods for block data type checking:
  - `is_stream_info`
  - `is_padding`
  - `is_application`
  - `is_seek_table`
  - `is_vorbis_comment`
  - `is_cue_sheet`
  - `is_picture`
  - `is_unknown`
* `ErrorKind` to public exports

### Changed

* Method field `length` is now private
* Metadata field `is_last` is now private in favor of the method
  `Method::is_last`
* Variant `PictureType::DuringPerformace` to
  `PictureType::DuringPerformance`, there was a missing "n"
* Around a 5% improvement on decode performance
* `get_stream_info`, `get_vorbis_comment`, `get_cue_sheet`, and
  `get_picture` to return a `flac::ErrorKind` on errors.
* `Stream::new`, `Stream::from_file`, and `Stream::from_buffer` to
  return a `flac::ErrorKind` on errors.

### Fixed

* Calculating of bits per sample for anything higher than 16.

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
