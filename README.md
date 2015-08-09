# FLAC

An implementation of [FLAC][flac], free lossless audio codec, written in
Rust.

## Implementation Status

The status of this FLAC implementation:

- [ ] Parser
  - [ ] metadata
    - [x] header
    - [ ] data
      - [x] stream info
      - [ ] padding
      - [ ] application
      - [ ] seek table
      - [ ] vorbis comment
      - [ ] cuesheet
      - [ ] picture
  - [ ] frame
    - [ ] header
    - [ ] footer
    - [ ] sub-frame
      - [ ] header
      - [ ] constant
      - [ ] fixed
      - [ ] LPC
      - [ ] verbatim
- [ ] decoder
- [ ] encoder

[flac]: https://xiph.org/flac
