use frame::ChannelAssignment;

pub fn decode_left_side(buffer: &mut [i32]) {
  let block_size = buffer.len() / 2;

  for i in 0..block_size {
    let left = buffer[i];
    let side = buffer[i + block_size];

    // right channel
    buffer[i + block_size] = left - side;
  }
}

fn decode_right_side(buffer: &mut [i32]) {
  let block_size = buffer.len() / 2;

  for i in 0..block_size {
    let side  = buffer[i];
    let right = buffer[i + block_size];

    // left channel
    buffer[i] = side + right;
  }
}

fn decode_middle_side(buffer: &mut [i32]) {
  let block_size = buffer.len() / 2;

  for i in 0..block_size {
    let mut middle = buffer[i];
    let side       = buffer[i + block_size];

    middle = (middle << 1) | (side & 1);

    // left and right channel
    buffer[i]              = (middle + side) >> 1;
    buffer[i + block_size] = (middle - side) >> 1;
  }
}

pub fn decode(channel_assignment: ChannelAssignment, buffer: &mut [i32]) {
  match channel_assignment {
    ChannelAssignment::Independent => return,
    ChannelAssignment::LeftSide    => decode_left_side(buffer),
    ChannelAssignment::RightSide   => decode_right_side(buffer),
    ChannelAssignment::MiddleSide  => decode_middle_side(buffer),
  }
}
