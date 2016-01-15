extern crate docopt;
extern crate flac;
extern crate rustc_serialize;

use docopt::Docopt;
use flac::{ReadStream, Stream, StreamProducer};
use flac::metadata;
use flac::metadata::VorbisComment;

use std::env;
use std::fs::File;

const USAGE: &'static str = "
Usage: metadata streaminfo [options] <input>
       metadata comments [options] <input>
       metadata --help

Options:
  --block-size       Show both the max and min block size from StreamInfo.
  --frame-size       Show both the max and min frame size from StreamInfo.
  --sample-rate      Show the sample rate from StreamInfo.
  --channels         Show the number of channels from StreamInfo.
  --bits-per-sample  Show the size in bits for each sample from StreamInfo.
  --total-samples    Show total number of samples from StreamInfo.
  --md5              Show the MD5 signature from StreamInfo.
  --vendor           Show the vendor string from VorbisComment.
  --name=NAME        Show the comments matching the `NAME` from VorbisComment.
  -h, --help         Show this message.
";

#[derive(Debug, RustcDecodable)]
struct Arguments {
  arg_input: String,
  cmd_streaminfo: bool,
  cmd_comments: bool,
  flag_block_size: bool,
  flag_frame_size: bool,
  flag_sample_rate: bool,
  flag_channels: bool,
  flag_bits_per_sample: bool,
  flag_total_samples: bool,
  flag_md5: bool,
  flag_vendor: bool,
  flag_name: Option<String>,
}

fn print_stream_info<P>(stream: &Stream<P>, args: &Arguments)
 where P: StreamProducer {
  let info     = stream.info();
  let no_flags = (args.flag_block_size      || args.flag_frame_size    ||
                  args.flag_sample_rate     || args.flag_channels      ||
                  args.flag_bits_per_sample || args.flag_total_samples ||
                  args.flag_md5) == false;

  if no_flags || args.flag_block_size {
    println!("Minimum block size: {} samples", info.min_block_size);
    println!("Maximum block size: {} samples", info.max_block_size);
  }

  if no_flags || args.flag_frame_size {
    println!("Minimum frame size: {} bytes", info.min_frame_size);
    println!("Maximum frame size: {} bytes", info.max_frame_size);
  }

  if no_flags || args.flag_sample_rate {
    println!("Sample rate: {} Hz", info.sample_rate);
  }

  if no_flags || args.flag_channels {
    println!("Number of channels: {}", info.channels);
  }

  if no_flags || args.flag_bits_per_sample {
    println!("Bits per sample: {}", info.bits_per_sample);
  }

  if no_flags || args.flag_total_samples {
    println!("Total samples: {}", info.total_samples);
  }

  if no_flags || args.flag_md5 {
    let mut md5  = String::with_capacity(32);

    for byte in &info.md5_sum {
      let hex = format!("{:02x}", byte);

      md5.push_str(&hex);
    }

    println!("MD5 sum: {}", md5);
  }
}

fn print_vorbis_comments(vorbis_comment: &VorbisComment) {
  let mut index = 1;

  println!("Vendor String: {}", vorbis_comment.vendor_string);
  println!("Number of Comments: {}", vorbis_comment.comments.len());

  for comment in &vorbis_comment.comments {
    println!("  {}: \"{}\" = {}", index, comment.0, comment.1);

    index += 1;
  }
}

fn main() {
  let args: Arguments = Docopt::new(USAGE)
    .and_then(|d| d.argv(env::args()).decode())
    .unwrap_or_else(|e| e.exit());

  let stream = Stream::<ReadStream<File>>::from_file(&args.arg_input)
                 .expect("Couldn't parse file");

  if args.cmd_streaminfo {
    print_stream_info(&stream, &args);
  }

  for meta in stream.metadata() {
    match meta.data {
      metadata::Data::VorbisComment(ref v) => {
        if args.cmd_comments {
          print_vorbis_comments(v)
        }
      }
      _                                    => continue,
    }
  }
}
