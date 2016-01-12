extern crate docopt;
extern crate flac;
extern crate rustc_serialize;

use docopt::Docopt;
use flac::{ReadStream, Stream, StreamProducer};

use std::env;
use std::fs::File;

const USAGE: &'static str = "
Usage: metadata streaminfo <input>
       metadata --help

Options:
  -h, --help   Show this message.
";

#[derive(Debug, RustcDecodable)]
struct Arguments {
  arg_input: String,
  cmd_streaminfo: bool,
}

fn print_stream_info<P: StreamProducer>(stream: &Stream<P>) {
  let info = stream.info();

  println!("StreamInfo
  Minimum block size: {}
  Maximum block size: {}
  Maximum frame size: {}
  Maximum frame size: {}
  Sample rate: {} Hz
  Number of channels: {}
  Bits per sample: {}
  Total samples: {}
  MD5 sum: {:?}",
  info.min_block_size, info.max_block_size,
  info.min_frame_size, info.max_frame_size,
  info.sample_rate, info.channels, info.bits_per_sample, info.total_samples,
  info.md5_sum);
}

fn main() {
  let args: Arguments = Docopt::new(USAGE)
    .and_then(|d| d.argv(env::args()).decode())
    .unwrap_or_else(|e| e.exit());

  let stream = Stream::<ReadStream<File>>::from_file(&args.arg_input)
                 .expect("Couldn't parse file");

  if args.cmd_streaminfo {
    print_stream_info(&stream);
  }
}
