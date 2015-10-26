/// Maximum order of the fixed predictors permitted by the format.
pub const MAX_FIXED_ORDER: usize = 4;

/// Maximum LPC order permitted by the format.
pub const MAX_LPC_ORDER: usize   = 32;

/// A single channel of audio data.
pub struct SubFrame {
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
  /// An uncompressed suframe.
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
  pub method_type: CodingMethod,
  pub data: PartitionedRice,
}

/// The available entropy coding methods.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodingMethod {
  PartitionedRice,
  PartitionedRice2,
}

/// Header for a Rice partitioned residual.
#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedRice {
  pub order: u32,
  pub contents: PartitionedRiceContents,
}

/// Contents of a Rice partitioned residual.
#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedRiceContents {
  pub parameters: Vec<u32>,
  pub raw_bits: Vec<u32>,
}

impl PartitionedRiceContents {
  pub fn new(capacity: usize) -> PartitionedRiceContents {
    let mut parameters = Vec::with_capacity(capacity);
    let mut raw_bits   = Vec::with_capacity(capacity);

    unsafe {
      parameters.set_len(capacity);
      raw_bits.set_len(capacity);
    }

    PartitionedRiceContents {
      parameters: parameters,
      raw_bits: raw_bits,
    }
  }
}
