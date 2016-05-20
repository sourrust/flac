# FLAC

[![Build Status](https://travis-ci.org/sourrust/flac.svg?branch=master)](https://travis-ci.org/sourrust/flac)

An implementation of [FLAC][flac], free lossless audio codec, written in
Rust.

[Documentation][documentation]

## Install

flac is on [crates.io][crates] and can be included in your Cargo file
like so:

```toml
[dependencies]

flac = "^0.4.0"
```

Followed by including it in you code:

```rust
extern crate flac;
```

## Implementation Status

The status of this FLAC implementation:

Currently this project fully parses every FLAC file I've thrown at it
and the decoder is working great for any file that has a bit sample size
of 16 and before. This is based on the test suite I have on this project
and the tests do fail when the bit sample size is larger than 16.

Now that I have the varied size integers, making the buffer allocation
more efficient, I want to start on the encoding side of FLAC. It will be
a bit slower as I am busy with work but that is a goal of the project
for sure.

- [ ] serialization
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
  - [ ] frame
    - [ ] header
    - [ ] footer
    - [ ] sub-frame
      - [ ] header
      - [ ] constant
      - [ ] fixed
      - [ ] LPC
      - [ ] verbatim
- [ ] encoder
  - [ ] frame
    - [ ] left side
    - [ ] right side
    - [ ] midpoint side
  - [ ] sub-frame
    - [ ] fixed
    - [ ] LPC

[flac]: https://xiph.org/flac
[documentation]: https://sourrust.github.io/flac
[crates]: https://crates.io/crates/flac/
