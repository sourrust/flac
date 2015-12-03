mod crc;
#[macro_use]
mod macros;

pub use self::crc::{crc8, crc16};

use nom::{FileProducer, Move, Needed};

// Convert one to four byte slices into an unsigned 32-bit number.
//
// NOTE: This assumes big-endian since most numbers in the FLAC binary are
// that endianness.
#[inline]
pub fn to_u32(bytes: &[u8]) -> u32 {
  let length = bytes.len();

  debug_assert!(length <= 4);

  (0..length).fold(0, |result, i|
    result + ((bytes[i] as u32) << ((length - 1 - i) * 8))
  )
}

pub fn extend_sign(value: u32, bit_count: usize) -> i32 {
  if bit_count >= 32 || value < (1 << (bit_count - 1)) {
    value as i32
  } else {
    (value as i32).wrapping_sub(1 << bit_count)
  }
}

pub fn resize_producer(producer: &mut FileProducer,
                       await: &Move,
                       current_size: usize)
                       -> Option<usize> {
  let mut result = None;

  if let Move::Await(needed) = *await {
    if let Needed::Size(size) = needed {
      if size > current_size {
        producer.resize(size);
        result = Some(size);
      }
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[should_panic]
  fn test_panic_to_u32() {
    to_u32(&[0x00, 0x00, 0x00, 0x00, 0x00]);
  }

  #[test]
  fn test_to_u32() {
    let bytes = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];

    assert_eq!(to_u32(&bytes[0..1]), 0x00000001);
    assert_eq!(to_u32(&bytes[3..5]), 0x00006789);
    assert_eq!(to_u32(&bytes[1..4]), 0x00234567);
    assert_eq!(to_u32(&bytes[4..]), 0x89abcdef);
  }

  #[test]
  fn test_extend_sign() {
    assert_eq!(extend_sign(32, 6), -32);
    assert_eq!(extend_sign(31, 6), 31);
    assert_eq!(extend_sign(128, 8), -128);
    assert_eq!(extend_sign(127, 8), 127);

    assert_eq!(extend_sign(2048, 12), -2048);
    assert_eq!(extend_sign(2047, 12), 2047);
    assert_eq!(extend_sign(32768, 16), -32768);
    assert_eq!(extend_sign(32767, 16), 32767);

    assert_eq!(extend_sign(8388608, 24), -8388608);
    assert_eq!(extend_sign(8388607, 24), 8388607);
    assert_eq!(extend_sign(2147483648, 32), -2147483648);
    assert_eq!(extend_sign(2147483647, 32), 2147483647);
  }
}
