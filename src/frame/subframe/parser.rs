use nom::{
  IResult,
  ErrorCode, Err,
};

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
