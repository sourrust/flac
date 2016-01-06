# FLAC

[![Build Status](https://travis-ci.org/sourrust/flac.svg?branch=master)](https://travis-ci.org/sourrust/flac)

An implementation of [FLAC][flac], free lossless audio codec, written in
Rust.

[Documentation][documentation]

## Install

flac is not on crates.io at the moment, but you can include it in your
Cargo file like so:

```toml
[dependencies.flac]

git = "https://github.com/sourrust/flac.git"

```

Followed by including it in you code:

```rust
extern crate flac;
```

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
- [x] decoder
  - [x] frame
    - [x] left side
    - [x] right side
    - [x] middle side
  - [x] sub-frame
    - [x] fixed restoration
    - [x] LPC restoration
- [ ] encoder

[flac]: https://xiph.org/flac
[documentation]: https://sourrust.github.io/flac
