# FLAC

An implementation of [FLAC][flac], free lossless audio codec, written in
Rust.

## Implementation Status

The status of this FLAC implementation:

- [ ] Parser
  - [ ] metadata
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
- [ ] decoder
- [ ] encoder

[flac]: https://xiph.org/flac
