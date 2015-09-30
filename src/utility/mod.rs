mod crc;

pub use self::crc::{crc8, crc16};

// Convert one to four byte slices in an unsigned 32-bit number.
//
// NOTE: This assumes big-endian since most numbers in the FLAC binary are
// that endianness.
#[inline]
pub fn to_u32(bytes: &[u8]) -> u32 {
  let length = bytes.len();

  assert!(length <= 4);

  (0..length).fold(0, |result, i|
    result + ((bytes[i] as u32) << ((length - 1 - i) * 8))
  )
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
}
