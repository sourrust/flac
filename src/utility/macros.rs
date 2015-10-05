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
