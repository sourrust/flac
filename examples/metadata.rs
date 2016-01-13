extern crate docopt;
extern crate flac;
extern crate rustc_serialize;

use docopt::Docopt;
use flac::{ReadStream, Stream, StreamProducer};

use std::env;
use std::fs::File;

const USAGE: &'static str = "
Usage: metadata streaminfo [options] <input>
       metadata --help

Options:
  --block-size       Show both the max and min block size from StreamInfo.
  --frame-size       Show both the max and min frame size from StreamInfo.
  --sample-rate      Show the sample rate from StreamInfo.
  --channels         Show the number of channels from StreamInfo.
  --bits-per-sample  Show the size in bits for each sample from StreamInfo.
  --total-samples    Show total number of samples from StreamInfo.
  --md5              Show the MD5 signature from StreamInfo.
  -h, --help         Show this message.
";

#[derive(Debug, RustcDecodable)]
struct Arguments {
  arg_input: String,
  cmd_streaminfo: bool,
  flag_block_size: bool,
  flag_frame_size: bool,
  flag_sample_rate: bool,
  flag_channels: bool,
  flag_bits_per_sample: bool,
  flag_total_samples: bool,
  flag_md5: bool,
}

fn print_stream_info<P: StreamProducer>(stream: &Stream<P>) {
  let info    = stream.info();
  let mut md5 = String::with_capacity(32);

  for byte in &info.md5_sum {
    let hex = format!("{:02x}", byte);

    md5.push_str(&hex);
  }

  println!("StreamInfo
  Minimum block size: {} samples
  Maximum block size: {} samples
  Minimum frame size: {} bytes
  Maximum frame size: {} bytes
  Sample rate: {} Hz
  Number of channels: {}
  Bits per sample: {}
  Total samples: {}
  MD5 sum: {}",
  info.min_block_size, info.max_block_size,
  info.min_frame_size, info.max_frame_size,
  info.sample_rate, info.channels, info.bits_per_sample,
  info.total_samples, md5);
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
