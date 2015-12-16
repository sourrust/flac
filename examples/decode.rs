extern crate docopt;
extern crate flac;
extern crate hound;
extern crate nom;
extern crate rustc_serialize;

use docopt::Docopt;
use flac::Stream;
use std::env;
use std::error::Error;

const USAGE: &'static str = "
Usage: decode <input> <output>
       decode <input>... <dir>
       decode --help

Options:
  -h, --help   Show this message.
";

#[derive(RustcDecodable)]
struct Arguments {
  arg_input: Vec<String>,
  arg_output: Option<String>,
  arg_dir: Option<String>,
}

fn decode_file(input_file: &str, output_file: &str)
               -> Result<(), hound::Error> {
  let mut stream = try! {
    Stream::from_file(input_file).map_err(hound::Error::IoError)
  };
  let info       = stream.info();
  let frames_len = stream.frames.len();

  let spec = hound::WavSpec {
    channels: info.channels as u16,
    sample_rate: info.sample_rate,
    bits_per_sample: info.bits_per_sample as u16,
  };

  let mut output = try!(hound::WavWriter::create(output_file, spec));

  for _ in 0..frames_len {
    if let Some(buffer) = stream.next_frame() {
      let buffer_size = buffer.len();
      let block_size  = buffer_size / 2;
      let left        = &buffer[0..block_size];
      let right       = &buffer[block_size..buffer_size];

      for i in 0..block_size {
        try!(output.write_sample(left[i]));
        try!(output.write_sample(right[i]));
      }
    } else {
      break;
    }
  }

  output.finalize()
}

fn main() {
  let args: Arguments = Docopt::new(USAGE)
    .and_then(|d| d.argv(env::args()).decode())
    .unwrap_or_else(|e| e.exit());

  if let Some(ref output_file) = args.arg_output {
    let input_file = &args.arg_input[0];

    if let Err(e) = decode_file(input_file, output_file) {
      println!("{:?}", e);
    } else {
      println!("decoded: {} -> {}", input_file, output_file);
    }
  }
}
