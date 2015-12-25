extern crate flac;
extern crate crypto;

use crypto::digest::Digest;
use crypto::md5::Md5;
use flac::Stream;
use std::cmp;

fn to_bytes(value: i32, buffer: &mut [u8]) {
  buffer[0] = value as u8;
  buffer[1] = (value >> 8) as u8;
  buffer[2] = (value >> 16) as u8;
  buffer[3] = (value >> 24) as u8;
}

fn get_offset(sample_size: usize) -> usize {
  let bits_per_sample = cmp::max(sample_size, 8);

  bits_per_sample / 8
}

#[test]
fn test_decoded_md5_sum() {
  let filenames = [
    "tests/assets/input-pictures.flac",
    "tests/assets/input-SCPAP.flac",
    "tests/assets/input-SVAUP.flac",
  ];

  let mut buffer  = [0; 4];
  let mut md5     = Md5::new();
  let mut md5_sum = [0; 16];

  {
    let mut stream = Stream::from_file(filenames[0]).unwrap();

    let info   = stream.info();
    let offset = get_offset(info.bits_per_sample as usize);

    for sample in stream.iter() {
      to_bytes(sample, &mut buffer);

      md5.input(&buffer[0..offset]);
    }

    md5.result(&mut md5_sum);

    assert_eq!(md5_sum, info.md5_sum);
  }

  md5.reset();

  {
    let mut stream = Stream::from_file(filenames[1]).unwrap();

    let info   = stream.info();
    let offset = get_offset(info.bits_per_sample as usize);

    for sample in stream.iter() {
      to_bytes(sample, &mut buffer);

      md5.input(&buffer[0..offset]);
    }

    md5.result(&mut md5_sum);

    assert_eq!(md5_sum, info.md5_sum);
  }
}
