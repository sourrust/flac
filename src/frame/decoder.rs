use frame::ChannelAssignment;

pub fn decode(channel_assignment: ChannelAssignment, buffer: &mut [i32]) {
  match channel_assignment {
    ChannelAssignment::Independent => return,
    ChannelAssignment::LeftSide    => unimplemented!(),
    ChannelAssignment::RightSide   => unimplemented!(),
    ChannelAssignment::MiddleSide  => unimplemented!(),
  }
}
