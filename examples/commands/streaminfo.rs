use std::fs::File;

use flac::{Stream, StreamProducer, StreamReader};

pub const USAGE: &'static str = "
Usage: metadata streaminfo [options] <filename>
       metadata streaminfo --help

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
pub struct Arguments {
  arg_filename: String,
  flag_block_size: bool,
  flag_frame_size: bool,
  flag_sample_rate: bool,
  flag_channels: bool,
  flag_bits_per_sample: bool,
  flag_total_samples: bool,
  flag_md5: bool,
}

fn print_stream_info<P>(stream: &Stream<P>, args: &Arguments)
 where P: StreamProducer {
  let info     = stream.info();
  let no_flags = (args.flag_block_size      || args.flag_frame_size    ||
                  args.flag_sample_rate     || args.flag_channels      ||
                  args.flag_bits_per_sample || args.flag_total_samples ||
                  args.flag_md5) == false;

  if no_flags || args.flag_block_size {
    let block_size_str = if info.is_fixed_block_size() {
      format!("{} samples", info.max_block_size)
    } else {
      format!("{} - {} samples", info.min_block_size, info.max_block_size)
    };

    format_print!("{}{}", "Block size: ", block_size_str, no_flags);
  }

  if no_flags || args.flag_frame_size {
    println!("Frame size: {} - {} bytes", info.min_frame_size,
                                          info.max_frame_size);
  }

  if no_flags || args.flag_sample_rate {
    format_print!("{}{} Hz", "Sample rate: ", info.sample_rate, no_flags);
  }

  if no_flags || args.flag_channels {
    format_print!("{}{}", "Number of channels: ", info.channels, no_flags);
  }

  if no_flags || args.flag_bits_per_sample {
    format_print!("{}{}", "Bits per samples: ", info.bits_per_sample,
                                                no_flags);
  }

  if no_flags || args.flag_total_samples {
    format_print!("{}{}", "Total samples: ", info.total_samples, no_flags);
  }

  if no_flags || args.flag_md5 {
    let mut md5  = String::with_capacity(32);

    for byte in &info.md5_sum {
      let hex = format!("{:02x}", byte);

      md5.push_str(&hex);
    }

    format_print!("{}{}", "MD5 sum: ", md5, no_flags);
  }
}

pub fn run(args: &Arguments) {
  let stream = StreamReader::<File>::from_file(&args.arg_filename)
                 .expect("Couldn't parse file");

  print_stream_info(&stream, &args);
}
