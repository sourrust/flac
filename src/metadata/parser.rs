use nom::be_u8;

use utility::to_u32;

named!(header <&[u8], (u8, bool, u32)>,
  chain!(
    block_byte: be_u8 ~
    length: map!(take!(3), to_u32),
    || {
      let is_last    = (block_byte >> 7) == 1;
      let block_type = block_byte & 0b01111111;

      (block_type, is_last, length)
    }
  )
);
