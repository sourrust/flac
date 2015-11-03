use subframe;
use subframe::Subframe;

pub fn decode(subframe: &Subframe, output: &mut [i32]) {
  match subframe.data {
    subframe::Data::Constant(_) => unimplemented!(),
    subframe::Data::Verbatim(_) => unimplemented!(),
    subframe::Data::Fixed(_)    => unimplemented!(),
    subframe::Data::LPC(_)      => unimplemented!(),
  }

  if subframe.wasted_bits > 0 {
    for value in output {
      *value <<= subframe.wasted_bits;
    }
  }
}
