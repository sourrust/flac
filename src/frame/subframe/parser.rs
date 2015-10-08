use nom::{
  IResult,
  ErrorCode, Err,
  Needed,
};

use frame;
use frame::{subframe, ChannelAssignment};
use frame::SubFrame;
use frame::subframe::{CodingMethod, PartitionedRiceContents};

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

fn adjust_bits_per_sample(frame_header: &frame::Header,
                          channel: usize)
                          -> usize {
  let bits_per_sample = frame_header.bits_per_sample;

  match frame_header.channel_assignment {
    // Independent doesn't adjust bits per sample.
    ChannelAssignment::Independent => bits_per_sample,
    ChannelAssignment::LeftSide    => {
      if channel == 1 {
        bits_per_sample + 1
      } else {
        bits_per_sample
      }
    }
    ChannelAssignment::RightSide   => {
      if channel == 0 {
        bits_per_sample + 1
      } else {
        bits_per_sample
      }
    }
    ChannelAssignment::MiddleSide  => {
      if channel == 1 {
        bits_per_sample + 1
      } else {
        bits_per_sample
      }
    }
  }
}

pub fn subframe_parser<'a>(input: (&'a [u8], usize),
                           frame_header: &frame::Header)
                           -> IResult<'a, (&'a [u8], usize), SubFrame> {
  chain!(input,
    subframe_header: header ~
    wasted_bits: map!(
      cond!(subframe_header.1, leading_zeros),
      |option: Option<u32>| option.map_or(0, |zeros| zeros + 1)
    ) ~
    subframe_data: apply!(data, frame_header, subframe_header.0 as usize,
                          wasted_bits as usize),
    || {
      SubFrame {
        data: subframe_data,
        wasted_bits: wasted_bits,
      }
    }
  )
}

fn header(input: (&[u8], usize)) -> IResult<(&[u8], usize), (usize, bool)> {
  match take_bits!(input, u8, 8) {
    IResult::Done(i, byte)    => {
      let is_valid        = (byte >> 7) == 0;
      let subframe_type   = (byte >> 1) & 0b111111;
      let has_wasted_bits = (byte & 0b01) == 1;

      if is_valid {
        IResult::Done(i, (subframe_type as usize, has_wasted_bits))
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input.0))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn data<'a>(input: (&'a [u8], usize),
            frame_header: &frame::Header,
            subframe_type: usize,
            wasted_bits: usize)
            -> IResult<'a, (&'a [u8], usize), subframe::Data> {
  let bits_per_sample = frame_header.bits_per_sample - wasted_bits;
  let block_size      = frame_header.block_size as usize;

  match subframe_type {
    0b000000            => constant(input, bits_per_sample),
    0b000001            => verbatim(input, bits_per_sample, block_size),
    0b001000...0b001100 => fixed(input, subframe_type & 0b0111,
                                 bits_per_sample, block_size),
    0b100000...0b111111 => lpc(input, (subframe_type & 0b011111) + 1,
                               bits_per_sample, block_size),
    _                   => IResult::Error(Err::Position(
                             ErrorCode::Alt as u32, input.0))
  }
}

fn constant(input: (&[u8], usize), bits_per_sample: usize)
            -> IResult<(&[u8], usize), subframe::Data> {
  map!(input, take_bits!(i32, bits_per_sample), subframe::Data::Constant)
}

fn fixed(input: (&[u8], usize),
         order: usize,
         bits_per_sample: usize,
         block_size: usize)
         -> IResult<(&[u8], usize), subframe::Data> {
  let mut warmup = [0; subframe::MAX_FIXED_ORDER];

  chain!(input,
    count_slice!(take_bits!(i32, bits_per_sample), &mut warmup[0..order]) ~
    tuple: apply!(residual, order, block_size),
    || {
      let data = tuple;

      subframe::Data::Fixed(subframe::Fixed {
        entropy_coding_method: data.0,
        order: order as u8,
        warmup: warmup,
        residual: data.1,
      })
    }
  )
}

fn qlp_coefficient_precision(input: (&[u8], usize))
                             -> IResult<(&[u8], usize), u8> {
  match take_bits!(input, u8, 4) {
    IResult::Done(i, precision) => {
      if precision == 0b1111 {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input.0))
      } else {
        IResult::Done(i, precision + 1)
      }
    }
    IResult::Error(error)       => IResult::Error(error),
    IResult::Incomplete(need)   => IResult::Incomplete(need),
  }
}

fn lpc(input: (&[u8], usize),
       order: usize,
       bits_per_sample: usize,
       block_size: usize)
       -> IResult<(&[u8], usize), subframe::Data> {
  let mut warmup           = [0; subframe::MAX_LPC_ORDER];
  let mut qlp_coefficients = [0; subframe::MAX_LPC_ORDER];

  chain!(input,
    count_slice!(take_bits!(i32, bits_per_sample), &mut warmup[0..order]) ~
    qlp_coeff_precision: qlp_coefficient_precision ~
    quantization_level: take_bits!(i8, 5) ~
    count_slice!(
      take_bits!(i32, qlp_coeff_precision as usize),
      &mut qlp_coefficients[0..order]
    ) ~
    tuple: apply!(residual, order, block_size),
    || {
      let data = tuple;

      subframe::Data::LPC(subframe::LPC {
        entropy_coding_method: data.0,
        order: order as u8,
        qlp_coeff_precision: qlp_coeff_precision,
        quantization_level: quantization_level,
        qlp_coefficients: qlp_coefficients,
        warmup: warmup,
        residual: data.1,
      })
    }
  )
}

fn verbatim(input: (&[u8], usize), bits_per_sample: usize, block_size: usize)
            -> IResult<(&[u8], usize), subframe::Data> {
  // TODO: Use nom's `count!` macro as soon as it is fixed for bit parsers.
  map!(input, count_bits!(take_bits!(i32, bits_per_sample), block_size),
       subframe::Data::Verbatim)
}

fn coding_method(input: (&[u8], usize))
                 -> IResult<(&[u8], usize), CodingMethod> {
  match take_bits!(input, u8, 2) {
    IResult::Done(i, method)  => {
      match method {
        0 => IResult::Done(i, CodingMethod::PartitionedRice),
        1 => IResult::Done(i, CodingMethod::PartitionedRice2),
        _ => IResult::Error(Err::Position(ErrorCode::Alt as u32, input.0)),
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn residual(input: (&[u8], usize),
            predictor_order: usize,
            block_size: usize)
            -> IResult<(&[u8], usize),
                       (subframe::EntropyCodingMethod, Vec<i32>)> {
  match pair!(input, coding_method, take_bits!(u32, 4)) {
    IResult::Done(i, data)    => {
      let (method, order) = data;

      rice_partition(i, order, predictor_order, block_size, method)
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn rice_partition(input: (&[u8], usize),
                  partition_order: u32,
                  predictor_order: usize,
                  block_size: usize,
                  method: CodingMethod)
                  -> IResult<(&[u8], usize),
                             (subframe::EntropyCodingMethod, Vec<i32>)> {
  let (param_size, escape_code) = match method {
    CodingMethod::PartitionedRice  => (4, 0b1111),
    CodingMethod::PartitionedRice2 => (5, 0b11111),
  };

  // Adjust block size to not include allocation for warm up samples
  let residual_size = block_size - predictor_order;

  let mut mut_input = input;
  let mut residual  = Vec::with_capacity(residual_size);
  let mut sample    = 0;
  let mut contents  = PartitionedRiceContents::new(partition_order);

  unsafe { residual.set_len(residual_size) }

  let partitions = 2_usize.pow(partition_order);

  for partition in 0..partitions {
    let offset = if partition_order == 0 {
      block_size - predictor_order
    } else if partition > 0 {
      block_size / partitions
    } else {
      (block_size / partitions) - predictor_order
    };
    let start = sample;
    let end   = sample + offset;

    let result = chain!(mut_input,
      rice_parameter: take_bits!(u32, param_size) ~
      size: cond!(rice_parameter == escape_code, take_bits!(usize, 5)) ~
      data: apply!(residual_data,
        size, rice_parameter,
        &mut contents.raw_bits[partition],
        &mut residual[start..end]
      ),
      || { rice_parameter }
    );

    match result {
      IResult::Done(i, parameter) => {
        mut_input = i;
        sample    = end;

        contents.parameters[partition] = parameter;
      }
      IResult::Error(error)       => return IResult::Error(error),
      IResult::Incomplete(need)   => return IResult::Incomplete(need),
    }
  }

  let entropy_coding_method = subframe::EntropyCodingMethod {
    method_type: method,
    data: subframe::PartitionedRice {
      order: partition_order,
      contents: contents,
    },
  };

  IResult::Done(mut_input, (entropy_coding_method, residual))
}

fn residual_data<'a>(input: (&'a [u8], usize),
                     option: Option<usize>,
                     rice_parameter: u32,
                     raw_bit: &mut u32,
                     samples: &mut [i32])
                     -> IResult<'a, (&'a [u8], usize), ()> {
  if let Some(size) = option {
    unencoded_residuals(input, size, raw_bit, samples)
  } else {
    encoded_residuals(input, rice_parameter, raw_bit, samples)
  }
}

fn unencoded_residuals<'a>(input: (&'a [u8], usize),
                           bits_per_sample: usize,
                           raw_bit: &mut u32,
                           samples: &mut [i32])
                           -> IResult<'a, (&'a[u8], usize), ()> {
  *raw_bit = bits_per_sample as u32;

  count_slice!(input, take_bits!(i32, bits_per_sample), &mut samples[..])
}

fn encoded_residuals<'a>(input: (&'a [u8], usize),
                         parameter: u32,
                         raw_bit: &mut u32,
                         samples: &mut [i32])
                         -> IResult<'a, (&'a[u8], usize), ()> {
  let length = samples.len();

  let mut count     = 0;
  let mut is_error  = false;
  let mut mut_input = input;

  *raw_bit = 0;

  for sample in samples {
    let result = chain!(mut_input,
      quotient: leading_zeros ~
      remainder: take_bits!(u32, parameter as usize),
      || {
        let value = quotient * parameter + remainder;

        ((value as i32) >> 1) ^ -((value as i32) & 1)
      });

    match result {
      IResult::Done(i, value) => {
        mut_input = i;
        count    += 1;

        *sample = value
      }
      IResult::Error(_)       => {
        is_error = true;

        break;
      }
      IResult::Incomplete(_)  => break,
    }
  }

  if is_error {
    IResult::Error(Err::Position(ErrorCode::Count as u32, input.0))
  } else if count == length {
    IResult::Done(mut_input, ())
  } else {
    IResult::Incomplete(Needed::Unknown)
  }
}
