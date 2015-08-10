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
