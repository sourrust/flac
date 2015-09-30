macro_rules! skip_bytes (
  ($input: expr, $length: expr, $offset: expr) => (
    {
      match take!($input, $length) {
        IResult::Done(i, bytes)   => {
          let head        = bytes[0] << $offset;
          let tail        = &bytes[1..];
          let is_all_zero = tail.iter().all(|byte| *byte == 0);

          if head == 0 && is_all_zero {
            IResult::Done(i, bytes)
          } else {
            IResult::Error(Err::Position(ErrorCode::Digit as u32, $input))
          }
        }
        IResult::Error(error)     => IResult::Error(error),
        IResult::Incomplete(need) => IResult::Incomplete(need),
      }
    }
  );
);
