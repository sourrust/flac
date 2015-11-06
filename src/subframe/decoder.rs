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
}
