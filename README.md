# FLAC

[![Build Status](https://travis-ci.org/sourrust/flac.svg?branch=master)](https://travis-ci.org/sourrust/flac)

An implementation of [FLAC][flac], free lossless audio codec, written in
Rust.

[Documentation][documentation]

## Implementation Status

The status of this FLAC implementation:

- [x] Parser
  - [x] metadata
    - [x] header
    - [x] data
      - [x] stream info
      - [x] padding
      - [x] application
      - [x] seek table
      - [x] vorbis comment
      - [x] cuesheet
      - [x] picture
      - [x] unknown
  - [x] frame
    - [x] header
    - [x] footer
    - [x] sub-frame
      - [x] header
      - [x] constant
      - [x] fixed
      - [x] LPC
      - [x] verbatim
- [ ] decoder
- [ ] encoder

[flac]: https://xiph.org/flac
[documentation]: https://sourrust.github.io/flac
