use nom::{
  IResult,
  ErrorKind, Err,
  Needed,
};

use frame::{self, ChannelAssignment};
use subframe::{self, Subframe, CodingMethod, PartitionedRiceContents};
use utility::power_of_two;

// Parser used to parse unary notation. Naming the parser `leading_zeros`
// was something that felt more clear in the code. It actually tells the
// caller what the parser doing considering unary notation can -- and more
// commonly -- be leading ones.
pub fn leading_zeros(input: (&[u8], usize)) -> IResult<(&[u8], usize), u32> {
  let (bytes, mut offset) = input;

  let mut index     = 0;
  let mut count     = 0;
  let mut is_parsed = false;
  let bytes_len     = bytes.len();

  for i in 0..bytes_len {
    // Clear the number of offset bits
    let byte  = bytes[i] << offset;
    let zeros = byte.leading_zeros() as usize;

    index = i;

    if byte > 0 {
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
  } else if index + 2 > bytes_len {
    IResult::Incomplete(Needed::Size(index + 2))
  } else {
    IResult::Error(Err::Position(ErrorKind::TakeUntil, (bytes, offset)))
  }
}

// The channel's bits per sample that gets adjusted are the side channels
// for `LeftSide`, `MidpointSide`, and `RightSide`. The `Independent`
// channel assignment  doesn't get adjust on any of the channels.
pub fn adjust_bits_per_sample(frame_header: &frame::Header,
                              channel: usize)
                              -> usize {
  let bits_per_sample = frame_header.bits_per_sample;

  match frame_header.channel_assignment {
    ChannelAssignment::Independent  => bits_per_sample,
    ChannelAssignment::LeftSide     |
    ChannelAssignment::MidpointSide => {
      if channel == 1 {
        bits_per_sample + 1
      } else {
        bits_per_sample
      }
    }
    ChannelAssignment::RightSide    => {
      if channel == 0 {
        bits_per_sample + 1
      } else {
        bits_per_sample
      }
    }
  }
}

/// Parse a single channel of audio data.
pub fn subframe_parser<'a>(input: (&'a [u8], usize),
                           frame_header: &frame::Header,
                           channel: &mut usize,
                           buffer: &mut [i32])
                           -> IResult<(&'a [u8], usize), Subframe> {
  let block_size      = frame_header.block_size as usize;
  let bits_per_sample = adjust_bits_per_sample(frame_header, *channel);
  let start           = *channel * block_size;
  let end             = (*channel + 1) * block_size;
  let buffer_slice    = &mut buffer[start..end];

  chain!(input,
    subframe_header: header ~
    wasted_bits: map!(
      cond!(subframe_header.1, leading_zeros),
      |option: Option<u32>| option.map_or(0, |zeros| zeros + 1)
    ) ~
    subframe_data: apply!(data,
      bits_per_sample - (wasted_bits as usize),
      block_size, subframe_header.0,
      buffer_slice),
    || {
      // Iterate over the current channel being parsed. This probably should
      // be abstracted away, but for now this is the solution.
      *channel += 1;

      Subframe {
        data: subframe_data,
        wasted_bits: wasted_bits,
      }
    }
  )
}

// Parses the first byte of the subframe. The first bit must be zero to
// prevent sync-fooling, next six bits determines the subframe data type.
// Last bit is is there is wasted bits per sample, value one being true.
pub fn header(input: (&[u8], usize))
              -> IResult<(&[u8], usize), (usize, bool)> {
  let (i, byte) = try_parse!(input, take_bits!(u8, 8));

  let is_valid        = (byte >> 7) == 0;
  let subframe_type   = (byte >> 1) & 0b111111;
  let has_wasted_bits = (byte & 0b01) == 1;

  if is_valid {
    IResult::Done(i, (subframe_type as usize, has_wasted_bits))
  } else {
    IResult::Error(Err::Position(ErrorKind::Digit, input))
  }
}

fn data<'a>(input: (&'a [u8], usize),
            bits_per_sample: usize,
            block_size: usize,
            subframe_type: usize,
            buffer: &mut [i32])
            -> IResult<(&'a [u8], usize), subframe::Data> {
  match subframe_type {
    0b000000            => constant(input, bits_per_sample),
    0b000001            => verbatim(input, bits_per_sample, block_size),
    0b001000...0b001100 => fixed(input, subframe_type & 0b0111,
                                 bits_per_sample, block_size, buffer),
    0b100000...0b111111 => lpc(input, (subframe_type & 0b011111) + 1,
                               bits_per_sample, block_size, buffer),
    _                   => IResult::Error(Err::Position(
                             ErrorKind::Alt, input))
  }
}

pub fn constant(input: (&[u8], usize), bits_per_sample: usize)
                -> IResult<(&[u8], usize), subframe::Data> {
  map!(input, take_signed_bits!(bits_per_sample), subframe::Data::Constant)
}

pub fn fixed<'a>(input: (&'a [u8], usize),
                 order: usize,
                 bits_per_sample: usize,
                 block_size: usize,
                 buffer: &mut [i32])
                 -> IResult<(&'a [u8], usize), subframe::Data> {
  let mut warmup = [0; subframe::MAX_FIXED_ORDER];

  chain!(input,
    count_slice!(take_signed_bits!(bits_per_sample), &mut warmup[0..order]) ~
    entropy_coding_method: apply!(residual, order, block_size, buffer),
    || {
      subframe::Data::Fixed(subframe::Fixed {
        entropy_coding_method: entropy_coding_method,
        order: order as u8,
        warmup: warmup,
        residual: Vec::new(),
      })
    }
  )
}

// This parser finds the bit length for each quantized linear predictor
// coefficient. To preven sync fooling, four bit value cant be all onces.
fn qlp_coefficient_precision(input: (&[u8], usize))
                             -> IResult<(&[u8], usize), u8> {
  let (i, precision) = try_parse!(input, take_bits!(u8, 4));

  if precision == 0b1111 {
    IResult::Error(Err::Position(ErrorKind::Digit, input))
  } else {
    IResult::Done(i, precision + 1)
  }
}

pub fn lpc<'a>(input: (&'a [u8], usize),
               order: usize,
               bits_per_sample: usize,
               block_size: usize,
               buffer: &mut [i32])
               -> IResult<(&'a [u8], usize), subframe::Data> {
  let mut warmup           = [0; subframe::MAX_LPC_ORDER];
  let mut qlp_coefficients = [0; subframe::MAX_LPC_ORDER];

  chain!(input,
    count_slice!(take_signed_bits!(bits_per_sample), &mut warmup[0..order]) ~
    qlp_coeff_precision: qlp_coefficient_precision ~
    quantization_level: take_signed_bits!(i8, 5) ~
    count_slice!(
      take_signed_bits!(qlp_coeff_precision as usize),
      &mut qlp_coefficients[0..order]
    ) ~
    entropy_coding_method: apply!(residual, order, block_size, buffer),
    || {
      subframe::Data::LPC(subframe::LPC {
        entropy_coding_method: entropy_coding_method,
        order: order as u8,
        qlp_coeff_precision: qlp_coeff_precision,
        quantization_level: quantization_level,
        qlp_coefficients: qlp_coefficients,
        warmup: warmup,
        residual: Vec::new(),
      })
    }
  )
}

pub fn verbatim(input: (&[u8], usize),
                bits_per_sample: usize,
                block_size: usize)
                -> IResult<(&[u8], usize), subframe::Data> {
  map!(input, count!(take_signed_bits!(bits_per_sample), block_size),
       subframe::Data::Verbatim)
}

// Parser for figuring out the partitioned Rice coding, which there are only
// two, and the parser with fail when value is greater than one.
fn coding_method(input: (&[u8], usize))
                 -> IResult<(&[u8], usize), CodingMethod> {
  let (i, method) = try_parse!(input, take_bits!(u8, 2));

  match method {
    0 => IResult::Done(i, CodingMethod::PartitionedRice),
    1 => IResult::Done(i, CodingMethod::PartitionedRice2),
    _ => IResult::Error(Err::Position(ErrorKind::Alt, input)),
  }
}

fn residual<'a>(input: (&'a [u8], usize),
                predictor_order: usize,
                block_size: usize,
                buffer: &mut [i32])
                -> IResult<(&'a [u8], usize), subframe::EntropyCodingMethod> {
  let (i, data) = try_parse!(input,
                    pair!(coding_method, take_bits!(u32, 4)));

  let (method, order) = data;

  rice_partition(i, order, predictor_order, block_size, method, buffer)
}

fn rice_partition<'a>(input: (&'a [u8], usize),
                      partition_order: u32,
                      predictor_order: usize,
                      block_size: usize,
                      method: CodingMethod,
                      buffer: &mut [i32])
                      -> IResult<(&'a [u8], usize),
                                 subframe::EntropyCodingMethod> {
  let (param_size, escape_code) = match method {
    CodingMethod::PartitionedRice  => (4, 0b1111),
    CodingMethod::PartitionedRice2 => (5, 0b11111),
  };

  // Adjust block size to not include allocation for warm up samples
  let partitions = power_of_two(partition_order) as usize;
  let residual   = &mut buffer[predictor_order..];

  let mut mut_input = input;
  let mut sample    = 0;
  let mut contents  = PartitionedRiceContents::new(partitions);

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
      apply!(residual_data,
        size, rice_parameter,
        &mut contents.raw_bits()[partition],
        &mut residual[start..end]
      ),
      || { rice_parameter }
    );

    match result {
      IResult::Done(i, parameter) => {
        mut_input = i;
        sample    = end;

        contents.parameters()[partition] = parameter;
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

  IResult::Done(mut_input, entropy_coding_method)
}

fn residual_data<'a>(input: (&'a [u8], usize),
                     option: Option<usize>,
                     rice_parameter: u32,
                     raw_bit: &mut u32,
                     samples: &mut [i32])
                     -> IResult<(&'a [u8], usize), ()> {
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
                           -> IResult<(&'a [u8], usize), ()> {
  *raw_bit = bits_per_sample as u32;

  count_slice!(input, take_signed_bits!(bits_per_sample), &mut samples[..])
}

fn encoded_residuals<'a>(input: (&'a [u8], usize),
                         parameter: u32,
                         raw_bit: &mut u32,
                         samples: &mut [i32])
                         -> IResult<(&'a [u8], usize), ()> {
  let length  = samples.len();
  let modulus = power_of_two(parameter);

  let mut count     = 0;
  let mut is_error  = false;
  let mut mut_input = input;

  *raw_bit = 0;

  for sample in samples {
    let result = chain!(mut_input,
      quotient: leading_zeros ~
      // TODO: Figure out the varied remainder bit size
      remainder: take_bits!(u32, parameter as usize),
      || {
        let value = quotient * modulus + remainder;

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
    IResult::Error(Err::Position(ErrorKind::Count, input))
  } else if count == length {
    IResult::Done(mut_input, ())
  } else {
    IResult::Incomplete(Needed::Unknown)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use nom::{
    IResult,
    Err, ErrorKind,
    Needed,
  };

  use frame;
  use frame::{ChannelAssignment, NumberType};
  use subframe::{
    Data,
    Fixed, LPC,
    EntropyCodingMethod, CodingMethod, PartitionedRice,
    PartitionedRiceContents,
  };

  #[test]
  fn test_leading_zeros() {
    let inputs  = [ (&[0b10000000][..], 0)
                  , (&[0b11000000][..], 1)
                  , (&[0b00000001][..], 0)
                  , (&[0b11111111][..], 7)
                  , (&[0b00000000, 0b10000000][..], 0)
                  , (&[0b10000000, 0b10000000][..], 1)
                  , (&[0b00000000, 0b00000001][..], 0)
                  , (&[0b11111110, 0b00000010][..], 7)
                  , (&[0b10101010, 0b00000000][..], 7)
                  ];
    let results = [ IResult::Done((&inputs[0].0[..], 1), 0)
                  , IResult::Done((&inputs[1].0[..], 2), 0)
                  , IResult::Done((&[][..], 0), 7)
                  , IResult::Done((&[][..], 0), 0)
                  , IResult::Done((&inputs[4].0[1..], 1), 8)
                  , IResult::Done((&inputs[5].0[1..], 1), 7)
                  , IResult::Done((&[][..], 0), 15)
                  , IResult::Done((&inputs[7].0[1..], 7), 7)
                  , IResult::Incomplete(Needed::Size(3))
                  ];

    assert_eq!(leading_zeros(inputs[0]), results[0]);
    assert_eq!(leading_zeros(inputs[1]), results[1]);
    assert_eq!(leading_zeros(inputs[2]), results[2]);
    assert_eq!(leading_zeros(inputs[3]), results[3]);
    assert_eq!(leading_zeros(inputs[4]), results[4]);
    assert_eq!(leading_zeros(inputs[5]), results[5]);
    assert_eq!(leading_zeros(inputs[6]), results[6]);
    assert_eq!(leading_zeros(inputs[7]), results[7]);
    assert_eq!(leading_zeros(inputs[8]), results[8]);
  }

  #[test]
  fn test_header() {
    let inputs  = [ (&[0b01010100][..], 0)
                  , (&[0b00011111][..], 0)
                  , (&[0b00000000][..], 0)
                  , (&[0b10000000][..], 0)
                  ];
    let results = [ IResult::Done((&[][..], 0), (0b101010, false))
                  , IResult::Done((&[][..], 0), (0b001111, true))
                  , IResult::Done((&[][..], 0), (0b000000, false))
                  , IResult::Error(Err::Position(ErrorKind::Digit, inputs[3]))
                  ];

    assert_eq!(header(inputs[0]), results[0]);
    assert_eq!(header(inputs[1]), results[1]);
    assert_eq!(header(inputs[2]), results[2]);
    assert_eq!(header(inputs[3]), results[3]);
  }

  #[test]
  fn test_adjust_bits_per_sample() {
    let mut frame_header = frame::Header {
      block_size: 512,
      sample_rate: 41000,
      channels: 2,
      channel_assignment: ChannelAssignment::Independent,
      bits_per_sample: 16,
      number: NumberType::Sample(40),
      crc: 0xc4,
    };

    assert_eq!(adjust_bits_per_sample(&frame_header, 0), 16);
    assert_eq!(adjust_bits_per_sample(&frame_header, 1), 16);

    frame_header.channel_assignment = ChannelAssignment::LeftSide;

    assert_eq!(adjust_bits_per_sample(&frame_header, 0), 16);
    assert_eq!(adjust_bits_per_sample(&frame_header, 1), 17);

    frame_header.channel_assignment = ChannelAssignment::RightSide;

    assert_eq!(adjust_bits_per_sample(&frame_header, 0), 17);
    assert_eq!(adjust_bits_per_sample(&frame_header, 1), 16);

    frame_header.channel_assignment = ChannelAssignment::MidpointSide;

    assert_eq!(adjust_bits_per_sample(&frame_header, 0), 16);
    assert_eq!(adjust_bits_per_sample(&frame_header, 1), 17);
  }

  #[test]
  fn test_constant() {
    let inputs  = [ (&b"\0\x80"[..], 0)
                  , (&b"\x18"[..], 3)
                  ];
    let results = [ IResult::Done((&[][..], 0), Data::Constant(128))
                  , IResult::Done((&[][..], 0), Data::Constant(-8))
                  ];

    assert_eq!(constant(inputs[0], 16), results[0]);
    assert_eq!(constant(inputs[1], 5), results[1]);
  }

  #[test]
  fn test_verbatim() {
    let inputs  = [ (&b"\xff\x80\0\x0a\xff\x65\0\0\x04\x28\xff\x28\
                        \0\0\xff\xe7"[..], 0)
                  , (&b"\xe2\x81\x07\x80\x89"[..], 0)
                  ];
    let results = [ IResult::Done((&[][..], 0), Data::Verbatim(vec![
                                  -128, 10, -155, 0, 1064, -216, 0, -25]))
                  , IResult::Done((&[][..], 0), Data::Verbatim(vec![
                                  -4, 10, 0, -16, 15, 0, 4, 9]))
                  ];

    assert_eq!(verbatim(inputs[0], 16, 8), results[0]);
    assert_eq!(verbatim(inputs[1], 5, 8), results[1]);
  }

  #[test]
  fn test_fixed() {
    let inputs  = [ (&b"\xe8\0\x40\xaf\x02\x01\x04\x80\x42\x92\x84\x65\
                        \x64"[..], 0)
                  , (&b"\xf5\x47\xf0\xff\xdc\0\x42\0\x8e\xf9\xbc\x08\x08\
                        \x10"[..], 0)
                  ];
    let results = [ IResult::Done((&[][..], 0), Data::Fixed(Fixed {
                      entropy_coding_method: EntropyCodingMethod {
                        method_type: CodingMethod::PartitionedRice,
                        data: PartitionedRice {
                          order: 0,
                          contents: PartitionedRiceContents {
                            parameters: vec![8],
                            raw_bits: vec![0],
                          },
                        },
                      },
                      order: 4,
                      warmup: [-24, 0, 64, -81],
                      residual: Vec::new(),
                    }))
                  , IResult::Done((&[][..], 0), Data::Fixed(Fixed {
                      entropy_coding_method: EntropyCodingMethod {
                        method_type: CodingMethod::PartitionedRice2,
                        data: PartitionedRice {
                          order: 1,
                          contents: PartitionedRiceContents {
                            parameters: vec![31, 31],
                            raw_bits: vec![16, 6],
                          },
                        },
                      },
                      order: 2,
                      warmup: [-1, 5, 0, 0],
                      residual: Vec::new(),
                    }))
                  ];

    let mut buffer = [0; 10];
    let residuals  = [ &[642, 0, 5, 148, -141, 178][..]
                     , &[-36, 66, 142, -4, 2, 0, -32, 16][..]
                     ];

    assert_eq!(fixed(inputs[0], 4, 8, 10, &mut buffer), results[0]);
    assert_eq!(&buffer[4..10], residuals[0]);

    assert_eq!(fixed(inputs[1], 2, 4, 10, &mut buffer), results[1]);
    assert_eq!(&buffer[2..10], residuals[1]);
  }

  #[test]
  fn test_lpc() {
    let inputs  = [ (&b"\xe8\0\x40\xaf\x74\x73\x19\0\x75\x81\xe8\x16\0\x05\
                        \x18\xef\x36"[..], 0)
                  , (&b"\x84\x01\xb6\xc2\x37\xf9\xd3\x82\x4a\xa2\x3b\xe9\xfc\
                        \x2b\x66\xea\x36\xcb\x85\x72\xc5\x13\x14\xed\x1b\
                        \x3f"[..], 0)
                  ];
    let slice = (&[27, 63][..], 2);
    let results = [ IResult::Done((&[][..], 0), Data::LPC(LPC {
                      entropy_coding_method: EntropyCodingMethod {
                        method_type: CodingMethod::PartitionedRice,
                        data: PartitionedRice {
                          order: 0,
                          contents: PartitionedRiceContents {
                            parameters: vec![15],
                            raw_bits: vec![8],
                          },
                        },
                      },
                      order: 4,
                      warmup: [ -24, 0, 64, -81, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                              , 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                              , 0, 0
                              ],
                      qlp_coeff_precision: 8,
                      quantization_level: 8,
                      qlp_coefficients: [ -26, 50, 0, -21, 0, 0, 0, 0, 0, 0, 0
                                        , 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                                        , 0, 0, 0, 0, 0, 0, 0, 0, 0
                                        ],
                      residual: Vec::new(),
                    }))
                  , IResult::Done(slice, Data::LPC(LPC {
                      entropy_coding_method: EntropyCodingMethod {
                        method_type: CodingMethod::PartitionedRice2,
                        data: PartitionedRice {
                          order: 1,
                          contents: PartitionedRiceContents {
                            parameters: vec![3, 5],
                            raw_bits: vec![0, 0],
                          },
                        },
                      },
                      order: 8,
                      warmup: [ -8, 4, 0, 1, -5, 6, -4, 2,  0, 0, 0, 0, 0, 0
                              , 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                              , 0, 0
                              ],
                      qlp_coeff_precision: 4,
                      quantization_level: 15,
                      qlp_coefficients: [ -1, 3, -6, 7, 0, 4, -7, 5, 0, 0, 0
                                        , 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                                        , 0, 0, 0, 0, 0, 0, 0, 0, 0
                                        ],
                      residual: Vec::new(),
                    }))
                  ];

    let mut buffer = [0; 26];
    let residuals  = [ &[22, 0, 5, 24, -17, 54][..],
                       &[ -2, 3, -1, -4, 2, 27, -28, 20, 11, 9, 12, -22, -3, 1
                        , 1, -25, -20, 26
                        ][..]
                     ];

    assert_eq!(lpc(inputs[0], 4, 8, 10, &mut buffer), results[0]);
    assert_eq!(&buffer[4..10], residuals[0]);

    assert_eq!(lpc(inputs[1], 8, 4, 26, &mut buffer), results[1]);
    assert_eq!(&buffer[8..26], residuals[1]);
  }
}
