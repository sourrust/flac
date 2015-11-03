use subframe;
use subframe::{Subframe, MAX_FIXED_ORDER};

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

pub fn decode(subframe: &Subframe, output: &mut [i32]) {
  match subframe.data {
    subframe::Data::Constant(_)      => unimplemented!(),
    subframe::Data::Verbatim(_)      => unimplemented!(),
    subframe::Data::Fixed(ref fixed) => {
      let order = fixed.order as usize;

      for i in 0..order {
        output[i] = fixed.warmup[i];
      }

      fixed_restore_signal(order, &fixed.residual, output);
    }
    subframe::Data::LPC(_)           => unimplemented!(),
  }

  if subframe.wasted_bits > 0 {
    for value in output {
      *value <<= subframe.wasted_bits;
    }
  }
}
