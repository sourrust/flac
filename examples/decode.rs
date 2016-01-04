extern crate docopt;
extern crate flac;
extern crate hound;
extern crate nom;
extern crate rustc_serialize;

use docopt::Docopt;
use flac::{Stream, ReadStream};

use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    Stream::<ReadStream<fs::File>>::from_file(input_file)
      .map_err(hound::Error::IoError)
  };

  let info = stream.info();
  let spec = hound::WavSpec {
    channels: info.channels as u16,
    sample_rate: info.sample_rate,
    bits_per_sample: info.bits_per_sample as u16,
  };

  let mut output = try!(hound::WavWriter::create(output_file, spec));

  for sample in &mut stream {
    try!(output.write_sample(sample));
  }

  output.finalize()
}

fn to_output_file(buffer: &mut PathBuf, path: &Path, directory: &str)
                  -> Result<(), hound::Error> {
  buffer.push(directory);

  path.file_name().map(|name| {
    buffer.push(name);
    buffer.set_extension("wav");
  }).ok_or_else(|| {
    let kind    = io::ErrorKind::NotFound;
    let message = "no file name found";
    let error   = io::Error::new(kind, message);

    hound::Error::IoError(error)
  })
}

fn decode_all_files(input_files: &Vec<String>, directory: &str)
                    -> Result<(), hound::Error> {
  let dir_path = Path::new(directory);

  if !dir_path.exists() {
    try!(fs::create_dir(dir_path).map_err(hound::Error::IoError))
  }

  for ref input_file in input_files {
    let mut buffer = PathBuf::new();
    let path       = Path::new(input_file);

    try!(to_output_file(&mut buffer, path, directory));

    let output_file = try! {
      buffer.to_str().ok_or_else(|| {
        let kind    = io::ErrorKind::InvalidInput;
        let message = "invalid unicode with file path";
        let error   = io::Error::new(kind, message);

        hound::Error::IoError(error)
      })
    };

    let result = decode_file(input_file, output_file);

    if result.is_ok() {
      println!("decoded: {} -> {}", input_file, output_file);
    } else {
      return result;
    }
  }

  Ok(())
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
  } else if let Some(ref directory) = args.arg_dir {
    if let Err(e) = decode_all_files(&args.arg_input, directory) {
      println!("{:?}", e);
    }
  }
}
