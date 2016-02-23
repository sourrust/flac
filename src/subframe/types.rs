/// Maximum order of the fixed predictors permitted by the format.
pub const MAX_FIXED_ORDER: usize = 4;

/// Maximum LPC order permitted by the format.
pub const MAX_LPC_ORDER: usize   = 32;

/// A single channel of audio data.
pub struct Subframe {
  /// Data containing one of the four different types of subframes.
  pub data: Data,
  /// Number of wasted bits within each sample.
  pub wasted_bits: u32,
}

/// General enum that holds all the different subframe data types.
#[derive(Debug, PartialEq, Eq)]
pub enum Data {
  /// A single value that represents a constant subframe.
  Constant(i32),
  /// An uncompressed subframe.
  Verbatim(Vec<i32>),
  /// Fixed linear prediction subframe.
  Fixed(Fixed),
  /// FIR linear prediction subframe.
  LPC(LPC),
}

/// Fixed linear prediction subframe.
#[derive(Debug, PartialEq, Eq)]
pub struct Fixed {
  /// Residual coding method.
  pub entropy_coding_method: EntropyCodingMethod,
  /// Polynomial order.
  pub order: u8,
  /// Samples used to warm up, or prime, the predictor.
  pub warmup: [i32; MAX_FIXED_ORDER],
  /// Remaining samples after the warm up samples.
  pub residual: Vec<i32>,
}

/// Finite Impulse Response (FIR) linear prediction subframe.
#[derive(Debug, PartialEq, Eq)]
pub struct LPC {
  /// Residual coding method.
  pub entropy_coding_method: EntropyCodingMethod,
  /// FIR order.
  pub order: u8,
  /// Quantized FIR filter coefficient precision in bits.
  pub qlp_coeff_precision: u8,
  /// Quantized linear predictor coefficient shift needed in bits.
  pub quantization_level: i8,
  /// FIR filter coefficients.
  pub qlp_coefficients: [i32; MAX_LPC_ORDER],
  /// Samples used to warm up, or prime, the predictor.
  pub warmup: [i32; MAX_LPC_ORDER],
  /// Remaining samples after the warm up samples.
  pub residual: Vec<i32>,
}

/// Header for the entropy coding method.
#[derive(Debug, PartialEq, Eq)]
pub struct EntropyCodingMethod {
  /// The type of coding method being used.
  pub method_type: CodingMethod,
  /// Data for each entropy coding method partition.
  pub data: PartitionedRice,
}

/// The available entropy coding methods.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodingMethod {
  /// Coding partition with a 4-bit Rice parameter.
  PartitionedRice,
  /// Coding partition with a 5-bit Rice parameter.
  PartitionedRice2,
}

/// Header for a Rice partitioned residual.
#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedRice {
  /// Partition order.
  pub order: u32,
  /// Rice parameters and/or raw bits.
  pub contents: PartitionedRiceContents,
}

/// Contents of a Rice partitioned residual.
#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedRiceContents {
  /// Size of `parameters` and `raw_bits` within the data buffer.
  pub capacity: usize,
  /// Data buffer containing both `parameters` and `raw_bits`.
  pub data: Vec<u32>,
}

impl PartitionedRiceContents {
  pub fn new(capacity: usize) -> PartitionedRiceContents {
    let full_capacity = capacity * 2;
    let mut data      = Vec::with_capacity(full_capacity);

    unsafe { data.set_len(full_capacity) }

    PartitionedRiceContents {
      capacity: capacity,
      data: data,
    }
  }
}
