// A parser combinator that scans for zeros in a number of bytes. When the
// current byte is all zeros, the parser fails. The last argument is the bit
// offset relative the first byte being parsed.
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
              $crate::nom::ErrorKind::Digit, $input))
          }
        }
        $crate::nom::IResult::Error(error)     =>
          $crate::nom::IResult::Error(error),
        $crate::nom::IResult::Incomplete(need) =>
          $crate::nom::IResult::Incomplete(need),
      }
    }
  );
  ($input: expr, $length: expr) => (
    skip_bytes!($input, $length, 0);
  );
);

// A parser combiner for previously allocated buffers that can be passed
// in as mutable slices. The macro will parse and fill the total length of
// the passed in slice.
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
          $crate::nom::ErrorKind::Count, $input))
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

// A parser combinator for converting parsed unsigned numbers into signed
// two's complement based numbers. Without an explicit type passed in, the
// default return type is `i32`.
macro_rules! take_signed_bits (
  ($input: expr, $signed_type: ty, $count: expr) => (
    map!($input, take_bits!(u32, $count), |value| {
      ::utility::extend_sign(value, $count) as $signed_type
    });
  );
  ($input: expr, $count: expr) => (
    take_signed_bits!($input, i32, $count);
  );
);

macro_rules! to_custom_error (
  ($error_type: ident) => (
    |_| $crate::nom::Err::Code($crate::nom::ErrorKind::Custom(
          ::utility::ErrorKind::$error_type))
  );
);
