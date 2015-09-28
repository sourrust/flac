use nom::{
  IResult,
  ErrorCode, Err,
  Needed,
};

fn leading_zeros(input: (&[u8], usize)) -> IResult<(&[u8], usize), u32> {
  let (bytes, mut offset) = input;

  let mut index     = 0;
  let mut count     = 0;
  let mut is_parsed = false;
  let bytes_len     = bytes.len();

  for i in 0..bytes_len {
    // Clear the number of offset bits
    let byte  = bytes[i] << offset;
    let zeros = byte.leading_zeros() as usize;

    if byte > 0 {
      index     = i;
      is_parsed = true;
      count    += zeros;
      offset   += zeros + 1;

      if offset >= 8 {
        index  += 1;
        offset -= 8;
      }

      break;
    } else {
      count += zeros - offset;
      offset = 0;
    }
  }

  if is_parsed {
    IResult::Done((&bytes[index..], offset), count as u32)
  } else if index + 1 > bytes_len {
    IResult::Incomplete(Needed::Size(index + 1))
  } else {
    IResult::Error(Err::Position(ErrorCode::TakeUntil as u32, bytes))
  }
}

fn header(input: (&[u8], usize)) -> IResult<(&[u8], usize), (u8, bool)> {
  let result = chain!(input,
    bit_padding: take_bits!(u8, 1) ~
    subframe_type: take_bits!(u8, 6) ~
    wasted_bit_flag: take_bits!(u8, 1),
    || {
      let is_valid        = bit_padding == 0;
      let has_wasted_bits = wasted_bit_flag == 1;

      (is_valid, subframe_type, has_wasted_bits)
    }
  );

  match result {
    IResult::Done(i, data)    => {
      let (is_valid, subframe_type, has_wasted_bits) = data;

      if is_valid {
        IResult::Done(i, (subframe_type, has_wasted_bits))
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input.0))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}
