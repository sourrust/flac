pub struct SubFrame {
  pub data: Data,
  pub wasted_bits: u32,
}

pub enum Data {
  Constant(i32),
  Verbatim(Vec<i32>),
}
