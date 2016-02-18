use std::io::{self, Write};
use std::fs::File;

use flac::stream::StreamReader;
use flac::metadata::{self, Picture};

pub const USAGE: &'static str = "
Usage: metadata picture [options] <filename>
       metadata picture --help

Options:
  --export=FILE      Export to file.
  --index=NUMBER     Index of the current metadata type.
  -h, --help         Show this message.
";

#[derive(Debug, RustcDecodable)]
pub struct Arguments {
  arg_filename: String,
  flag_export: Option<String>,
  flag_index: Option<usize>,
}

fn export_picture(picture: &Picture, filename: &str) -> io::Result<()> {
  File::create(filename).and_then(|mut file| file.write_all(&picture.data))
}

pub fn run(args: &Arguments) {
  let stream = StreamReader::<File>::from_file(&args.arg_filename)
                 .expect("Couldn't parse file");

  let mut index = 0;
  let end_index = args.flag_index.unwrap_or(0);

  for meta in stream.metadata() {
    match meta.data {
      metadata::Data::Picture(ref p) => {
        if index < end_index {
          index += 1;

          continue;
        }

        if let Some(ref filename) = args.flag_export {
          export_picture(p, filename).expect("couldn't write to file");

          break;
        }
      }
       _                             => continue,
    }
  }
}
