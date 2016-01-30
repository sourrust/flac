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
Usage: metadata streaminfo [options] <filename>
       metadata comments [options] <filename>
       metadata seektable <filename>
       metadata picture [options] <filename>
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
  --export=FILE      Export VorbisComment or Picture to file.
  --index=NUMBER     Index of the current metadata type.
  -h, --help         Show this message.
";

#[derive(Debug, RustcDecodable)]
struct Arguments {
  arg_filename: String,
  cmd_streaminfo: bool,
  cmd_comments: bool,
  cmd_seektable: bool,
  cmd_picture: bool,
  flag_block_size: bool,
  flag_frame_size: bool,
  flag_sample_rate: bool,
  flag_channels: bool,
  flag_bits_per_sample: bool,
  flag_total_samples: bool,
  flag_md5: bool,
  flag_vendor: bool,
  flag_name: Option<String>,
  flag_export: Option<String>,
  flag_index: Option<usize>,
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

fn print_stream_info<P>(stream: &Stream<P>, args: &Arguments)
 where P: StreamProducer {
  let info     = stream.info();
  let no_flags = (args.flag_block_size      || args.flag_frame_size    ||
                  args.flag_sample_rate     || args.flag_channels      ||
                  args.flag_bits_per_sample || args.flag_total_samples ||
                  args.flag_md5) == false;

  if no_flags || args.flag_block_size {
    let block_size_str = if info.min_block_size == info.max_block_size {
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

fn print_vorbis_comments(vorbis_comment: &VorbisComment, args: &Arguments) {
  let no_flags  = (args.flag_vendor || args.flag_name.is_some()) == false;

  if no_flags || args.flag_vendor {
    format_print!("{}{}", "Vendor string: ", vorbis_comment.vendor_string,
                                             no_flags);
  }

  if no_flags {
    let mut index = 1;

    println!("Number of Comments: {}", vorbis_comment.comments.len());

    for (name, value) in &vorbis_comment.comments {
      println!("  {}: \"{}\" = {}", index, name, value);

      index += 1;
    }
  } else {
    if let Some(ref name) = args.flag_name {
      vorbis_comment.comments.get(name)
                             .map(|value| println!("{}", value));
    }
  }
}

fn export_vorbis_comments(vorbis_comment: &VorbisComment, filename: &str)
                          -> io::Result<()> {
  let mut file = try!(File::create(filename));

  for (name, value) in &vorbis_comment.comments {
    try!(write!(file, "{}={}\n", name, value));
  }

  Ok(())
}

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
