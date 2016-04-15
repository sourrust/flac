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

flac = "^0.3.0"
```

Followed by including it in you code:

```rust
extern crate flac;
```

## Implementation Status

The status of this FLAC implementation:

- [ ] encoder

[flac]: https://xiph.org/flac
[documentation]: https://sourrust.github.io/flac
[crates]: https://crates.io/crates/flac/
