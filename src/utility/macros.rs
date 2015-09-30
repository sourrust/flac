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
