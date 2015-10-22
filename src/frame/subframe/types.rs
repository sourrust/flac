/// Maximum order of the fixed predictors permitted by the format.
pub const MAX_FIXED_ORDER: usize = 4;

/// Maximum LPC order permitted by the format.
pub const MAX_LPC_ORDER: usize   = 32;

/// A single channel of audio data.
pub struct SubFrame {
  pub data: Data,
  pub wasted_bits: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Data {
  Constant(i32),
  Verbatim(Vec<i32>),
  Fixed(Fixed),
  LPC(LPC),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Fixed {
  pub entropy_coding_method: EntropyCodingMethod,
  pub order: u8,
  pub warmup: [i32; MAX_FIXED_ORDER],
  pub residual: Vec<i32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LPC {
  pub entropy_coding_method: EntropyCodingMethod,
  pub order: u8,
  pub qlp_coeff_precision: u8,
  pub quantization_level: i8,
  pub qlp_coefficients: [i32; MAX_LPC_ORDER],
  pub warmup: [i32; MAX_LPC_ORDER],
  pub residual: Vec<i32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EntropyCodingMethod {
  pub method_type: CodingMethod,
  pub data: PartitionedRice,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodingMethod {
  PartitionedRice,
  PartitionedRice2,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedRice {
  pub order: u32,
  pub contents: PartitionedRiceContents,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedRiceContents {
  pub parameters: Vec<u32>,
  pub raw_bits: Vec<u32>,
  pub capacity_by_order: u32,
}

impl PartitionedRiceContents {
  pub fn new(capacity_by_order: u32) -> PartitionedRiceContents {
    let capacity       = 2_usize.pow(capacity_by_order);
    let mut parameters = Vec::with_capacity(capacity);
    let mut raw_bits   = Vec::with_capacity(capacity);

    unsafe {
      parameters.set_len(capacity);
      raw_bits.set_len(capacity);
    }

    PartitionedRiceContents {
      parameters: parameters,
      raw_bits: raw_bits,
      capacity_by_order: capacity_by_order,
    }
  }
}
