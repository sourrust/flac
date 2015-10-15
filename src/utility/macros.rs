macro_rules! skip_bytes (
  ($input: expr, $length: expr, $offset: expr) => (
    {
      match take!($input, $length) {
        $crate::nom::IResult::Done(i, bytes)   => {
          let head        = bytes[0] << $offset;
          let tail        = &bytes[1..];
          let is_all_zero = tail.iter().all(|byte| *byte == 0);

          if head == 0 && is_all_zero {
            $crate::nom::IResult::Done(i, bytes)
          } else {
            $crate::nom::IResult::Error($crate::nom::Err::Position(
              $crate::nom::ErrorCode::Digit as u32, $input))
          }
        }
        $crate::nom::IResult::Error(error)     =>
          $crate::nom::IResult::Error(error),
        $crate::nom::IResult::Incomplete(need) =>
          $crate::nom::IResult::Incomplete(need),
      }
    }
  );
);

macro_rules! count_slice (
  ($input: expr, $submac: ident!( $($args:tt)* ), $result: expr) => (
    {
      let mut input    = $input;
      let mut count    = 0;
      let mut is_error = false;

      for result in $result {
        match $submac!(input, $($args)*) {
          $crate::nom::IResult::Done(i, o)    => {
            *result = o;

            input   = i;
            count  += 1;
          }
          $crate::nom::IResult::Error(_)      => {
            is_error = true;
            break;
          }
          $crate::nom::IResult::Incomplete(_) => break,
        }
      }

      if is_error {
        $crate::nom::IResult::Error($crate::nom::Err::Position(
          $crate::nom::ErrorCode::Count as u32, $input.0))
      } else if count == $result.len() {
        $crate::nom::IResult::Done(input, ())
      } else {
        $crate::nom::IResult::Incomplete($crate::nom::Needed::Unknown)
      }
    }
  );
  ($i: expr, $f: expr, $count: expr) => (
    count_slice!($i, call!($f), $count);
  );
);

macro_rules! count_bits (
  ($input: expr, $submac: ident!( $($args:tt)* ), $count: expr) => (
    {
      let mut input    = $input;
      let mut count    = 0;
      let mut is_error = false;
      let mut result   = Vec::with_capacity($count);

      loop {
        if count == $count {
          break;
        }

        match $submac!(input, $($args)*) {
          $crate::nom::IResult::Done(i, o)    => {
            result.push(o);

            input  = i;
            count += 1;
          }
          $crate::nom::IResult::Error(_)      => {
            is_error = true;
            break;
          }
          $crate::nom::IResult::Incomplete(_) => break,
        }
      }

      if is_error {
        $crate::nom::IResult::Error($crate::nom::Err::Position(
          $crate::nom::ErrorCode::Count as u32, $input.0))
      } else if count == $count {
        $crate::nom::IResult::Done(input, result)
      } else {
        $crate::nom::IResult::Incomplete($crate::nom::Needed::Unknown)
      }
    }
  );
  ($i: expr, $f: expr, $count: expr) => (
    count_bits!($i, call!($f), $count);
  );
);

macro_rules! take_signed_bits (
  ($input: expr, $signed_type: ty, $count: expr) => (
    map!($input, take_bits!(u32, $count), |value| {
      let max_count = ::std::mem::size_of::<$signed_type>() * 8;

      if $count >= max_count || value < (1 << ($count - 1)) {
        value as $signed_type
      } else {
        (value as $signed_type).wrapping_sub(1 << $count)
      }
    });
  );
  ($input: expr, $count: expr) => (
    take_signed_bits!($input, i32, $count);
  );
);

#[cfg(test)]
mod tests {
  use super::*;
  use nom::{
    IResult,
    Err, ErrorCode
  };

  #[test]
  fn test_take_signed_bits() {
    let inputs      = [ (&[0b00100000][..], 2)
                      , (&[0b00011111][..], 2)
                      , (&[0b00001000, 0b00000000][..], 4)
                      , (&[0b00000111, 0b11111111][..], 4)
                      , (&[0b10000000, 0b00000000, 0b00000000][..], 0)
                      , (&[0b01111111, 0b11111111, 0b11111111][..], 0)
                      ];
    let results_i8  = [ IResult::Done((&[][..], 0), -32)
                      , IResult::Done((&[][..], 0), 31)
                      ];
    let results_i16 = [ IResult::Done((&[][..], 0), -2048)
                      , IResult::Done((&[][..], 0), 2047)
                      ];
    let results_i32 = [ IResult::Done((&[][..], 0), -8388608)
                      , IResult::Done((&[][..], 0), 8388607)
                      ];

    assert_eq!(take_signed_bits!(inputs[0], i8, 6), results_i8[0]);
    assert_eq!(take_signed_bits!(inputs[1], i8, 6), results_i8[1]);
    assert_eq!(take_signed_bits!(inputs[2], i16, 12), results_i16[0]);
    assert_eq!(take_signed_bits!(inputs[3], i16, 12), results_i16[1]);
    assert_eq!(take_signed_bits!(inputs[4], i32, 24), results_i32[0]);
    assert_eq!(take_signed_bits!(inputs[5], i32, 24), results_i32[1]);
  }
}
