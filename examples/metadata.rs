extern crate docopt;
extern crate flac;
extern crate rustc_serialize;

use docopt::Docopt;
use flac::{Stream, StreamProducer, StreamReader};
use flac::metadata::{self, Picture, SeekPoint, VorbisComment};

use std::env;
use std::io::{self, Write};
use std::fs::File;

const USAGE: &'static str = "
Usage: metadata <command> [<args>...]
       metadata seektable <filename>
       metadata picture [options] <filename>
       metadata --help

Options:
  --export=FILE      Export VorbisComment or Picture to file.
  --index=NUMBER     Index of the current metadata type.
  -h, --help         Show this message.
";

#[derive(Debug, RustcDecodable)]
struct Arguments {
  arg_filename: String,
  cmd_seektable: bool,
  cmd_picture: bool,
  flag_export: Option<String>,
  flag_index: Option<usize>,
  arg_command: Option<Command>,
  arg_args: Vec<String>,
}

macro_rules! format_print (
  ($format_str: expr, $opt_str: expr, $data: expr, $no_flag: expr) => (
    {
      println!($format_str, if $no_flag {
        $opt_str
      } else {
        ""
      }, $data);
    }
  );
);


fn print_seek_table(seek_points: &[SeekPoint]) {
  let mut count = 0;

  println!("Number of Seek Points: {}", seek_points.len());

  for seek_point in seek_points {
    println!("Seek Point #{}", count);
    println!("  Sample number: {}", seek_point.sample_number);
    println!("  Stream offset: {}", seek_point.stream_offset);
    println!("  Frame samples: {}", seek_point.frame_samples);
    count += 1;
  }
}

fn export_picture(picture: &Picture, filename: &str) -> io::Result<()> {
  File::create(filename).and_then(|mut file| file.write_all(&picture.data))
}

#[derive(Debug, RustcDecodable)]
enum Command {
  StreamInfo,
  Comments,
  SeekTable,
  Picture,
}

fn main() {
  let args: Arguments = Docopt::new(USAGE)
    .and_then(|d| d.argv(env::args()).decode())
    .unwrap_or_else(|e| e.exit());

  let mut index = 0;
  let _index    = args.flag_index.unwrap_or(0);

  let stream = StreamReader::<File>::from_file(&args.arg_filename)
                 .expect("Couldn't parse file");

  if args.cmd_streaminfo {
    print_stream_info(&stream, &args);
  }

  for meta in stream.metadata() {
    match meta.data {
      metadata::Data::VorbisComment(ref v) => {
        if args.cmd_comments {
          if let Some(ref filename) = args.flag_export {
            export_vorbis_comments(v, filename)
              .expect("couldn't write to file")
          } else {
            print_vorbis_comments(v, &args)
          }
        }
      }
      metadata::Data::SeekTable(ref s)     => {
        if args.cmd_seektable {
          print_seek_table(s);
        }
      }
      metadata::Data::Picture(ref p)       => {
        if args.cmd_picture {
          if index != _index {
            index += 1;

            continue;
          }

          if let Some(ref filename) = args.flag_export {
            export_picture(p, filename).expect("couldn't write to file");

            break;
          }
        }
      }
      _                                    => continue,
    }
  }
}
