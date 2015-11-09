use subframe;
use subframe::{Subframe, MAX_FIXED_ORDER, MAX_LPC_ORDER};

use std::ptr;

pub fn fixed_restore_signal(order: usize,
                            residual: &[i32],
                            output: &mut [i32]) {
  debug_assert!(order <= MAX_FIXED_ORDER);

  let polynomial = [ &[][..]
                   , &[1][..]
                   , &[-1, 2][..]
                   , &[1, -3, 3][..]
                   , &[-1, 4, -6, 4][..]
                   ];

  let coefficients = polynomial[order];

  for i in 0..residual.len() {
    let offset     = i + order;
    let prediction = coefficients.iter()
                      .zip(&output[i..offset])
                      .fold(0, |result, (coefficient, signal)|
                            result + coefficient * signal);


    output[offset] = residual[i] + prediction;
  }
}

pub fn lpc_restore_signal(quantization_level: i8,
                          coefficients: &[i32],
                          residual: &[i32],
                          output: &mut [i32]) {
  let order = coefficients.len();

  debug_assert!(order <= MAX_LPC_ORDER);

  for i in 0..residual.len() {
    let offset     = i + order;
    let prediction = coefficients.iter().rev()
                       .zip(&output[i..offset])
                       .fold(0, |result, (coefficient, signal)|
                             result + coefficient * signal);

    output[offset] = residual[i] + (prediction >> quantization_level);
  }
}

/// Decodes the current subframe.
///
/// * `Constant` - fills the length of `output` with the constant value
///   within the subframe.
/// * `Verbatim` - copies the data within the verbatim subframe over to
///   `output`.
/// * `Fixed` - restore the signal of the fixed linear prediction and put
///   the result into `output`.
/// * `LPC` - restore the signal of the finite impulse response linear
///   prediction and put the result into `output`.
pub fn decode(subframe: &Subframe, output: &mut [i32]) {
  match subframe.data {
    subframe::Data::Constant(constant)     => {
      for i in 0..output.len() {
        output[i] = constant
      }
    }
    subframe::Data::Verbatim(ref verbatim) => {
      let length = verbatim.len();

      unsafe {
        ptr::copy(verbatim.as_ptr(), output.as_mut_ptr(), length)
      }
    }
    subframe::Data::Fixed(ref fixed)       => {
      let order = fixed.order as usize;

      for i in 0..order {
        output[i] = fixed.warmup[i];
      }

      fixed_restore_signal(order, &fixed.residual, output);
    }
    subframe::Data::LPC(ref lpc)           => {
      let order        = lpc.order as usize;
      let coefficients = &lpc.qlp_coefficients[0..order];

      for i in 0..order {
        output[i] = lpc.warmup[i];
      }

      lpc_restore_signal(lpc.quantization_level, coefficients, &lpc.residual,
                         output);
    }
  }

  if subframe.wasted_bits > 0 {
    for value in output {
      *value <<= subframe.wasted_bits;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use subframe;
  use subframe::{
    Subframe, Fixed, LPC,
    EntropyCodingMethod, CodingMethod, PartitionedRice,
    PartitionedRiceContents,
  };

  #[test]
  fn test_fixed_restore_signal() {
    let residuals   = [ &[-19, -16, 17, -23, -7, 16, -16, -5, 3
                         , -8, -13, -15, -1][..]
                      , &[-6513][..]
                      ];
    let mut outputs = [ &mut [-729, -722, -667, 0, 0, 0, 0, 0, 0
                             , 0, 0, 0, 0, 0, 0, 0][..]
                      , &mut [21877, 27482, 0][..]
                      ];

    fixed_restore_signal(3, &residuals[0], &mut outputs[0]);
    fixed_restore_signal(2, &residuals[1], &mut outputs[1]);

    assert_eq!(&outputs[0], &[-729, -722, -667, -583, -486, -359, -225, -91
                             , 59, 209, 354, 497, 630, 740, 812, 845]);
    assert_eq!(&outputs[1], &[21877, 27482, 26574]);
  }

  #[test]
  fn test_lpc_restore_signal() {
    let residuals    = [ &[-2, -23, 14, 6, 3, -4, 12, -2, 10][..]
                       , &[3157][..]
                       ];
    let coefficients = [ &[1042, -399, -75, -269, 121, 166, -75][..]
                       , &[1757, -1199, 879, -836, 555, -255, 119][..]
                       ];
    let mut outputs = [ &mut [-796, -547, -285, -32, 199, 443, 670
                             , 0, 0, 0, 0, 0, 0, 0, 0, 0][..]
                      , &mut [-21363, -21951, -22649, -24364, -27297, -26870
                             ,-30017, 0][..]
                      ];

    lpc_restore_signal(9, &coefficients[0], &residuals[0], &mut outputs[0]);
    lpc_restore_signal(10, &coefficients[1], &residuals[1], &mut outputs[1]);

    assert_eq!(&outputs[0], &[-796, -547, -285, -32, 199, 443, 670, 875
                             , 1046, 1208, 1343, 1454, 1541, 1616, 1663
                             , 1701]);
    assert_eq!(&outputs[1], &[-21363, -21951, -22649, -24364, -27297, -26870
                             , -30017, -29718]);
  }

  #[test]
  fn test_decode() {
    let mut output = [0; 16];

    let constant = Subframe {
      data: subframe::Data::Constant(4),
      wasted_bits: 0,
    };

    let verbatim = Subframe {
      data: subframe::Data::Verbatim(vec![16, -3, 55, 49, -32, 6, 40, -90, 1
                                         ,0, 77, -12, 84, 10, -112, 136]),
      wasted_bits: 0,
    };

    let fixed = Subframe {
      data: subframe::Data::Fixed(Fixed {
        entropy_coding_method: EntropyCodingMethod {
          method_type: CodingMethod::PartitionedRice,
          data: PartitionedRice {
            order: 0,
            contents: PartitionedRiceContents {
              parameters: vec![],
              raw_bits: vec![],
            },
          },
        },
        order: 3,
        warmup: [-729, -722, -667, 0],
        residual: vec![-19, -16, 17, -23, -7, 16, -16, -5, 3 , -8, -13, -15
                      ,-1],
      }),
      wasted_bits: 0,
    };

    let lpc = Subframe {
      data: subframe::Data::LPC(LPC {
        entropy_coding_method: EntropyCodingMethod {
          method_type: CodingMethod::PartitionedRice,
          data: PartitionedRice {
            order: 0,
            contents: PartitionedRiceContents {
              parameters: vec![],
              raw_bits: vec![],
            },
          },
        },
        order: 7,
        qlp_coeff_precision: 0,
        quantization_level: 9,
        qlp_coefficients: [1042, -399, -75, -269, 121, 166, -75, 0, 0, 0, 0
                          ,0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                          ,0, 0, 0, 0],
        warmup: [-796, -547, -285, -32, 199, 443, 670, 0, 0, 0, 0, 0, 0, 0, 0
                ,0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        residual: vec![-2, -23, 14, 6, 3, -4, 12, -2, 10],
      }),
      wasted_bits: 0,
    };

    decode(&constant, &mut output);
    assert_eq!(&output, &[4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4]);

    decode(&verbatim, &mut output);
    assert_eq!(&output, &[16, -3, 55, 49, -32, 6, 40 , -90, 1, 0, 77, -12, 84
                         ,10 , -112, 136]);

    decode(&fixed, &mut output);
    assert_eq!(&output, &[-729, -722, -667, -583, -486, -359, -225, -91, 59
                         ,209, 354, 497, 630, 740, 812, 845]);

    decode(&lpc, &mut output);
    assert_eq!(&output, &[-796, -547, -285, -32, 199, 443, 670, 875, 1046
                         ,1208, 1343, 1454, 1541, 1616, 1663, 1701]);
  }

  #[test]
  fn test_wasted_bit_decode() {
    let mut output = [0; 4];

    let constant = Subframe {
      data: subframe::Data::Constant(1),
      wasted_bits: 10,
    };

    decode(&constant, &mut output);
    assert_eq!(&output, &[1024, 1024, 1024, 1024]);
  }
}
