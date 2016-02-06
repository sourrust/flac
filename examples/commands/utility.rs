use std::fs::File;

use flac::{StreamReader, metadata};

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

macro_rules! command (
  ($name: ident) => (
    {
      let args: $name::Arguments = Docopt::new($name::USAGE)
        .and_then(|d| d.argv(env::args()).decode())
        .unwrap_or_else(|e| e.exit());

      $name::run(&args)
    }
  );
);

pub fn list_block_names(filename: &str) {
  let stream = StreamReader::<File>::from_file(filename)
                 .expect("Couldn't parse file");

  // StreamInfo isn't in the metadata slice, so we put this first since it
  // is required to be the first metadata block.
  println!("stream info");

  for meta in stream.metadata() {
    println!("{}", match meta.data {
      metadata::Data::StreamInfo(_)    => "stream info",
      metadata::Data::Padding(_)       => "padding",
      metadata::Data::Application(_)   => "application",
      metadata::Data::SeekTable(_)     => "seek table",
      metadata::Data::VorbisComment(_) => "vorbis comment",
      metadata::Data::CueSheet(_)      => "cuesheet",
      metadata::Data::Picture(_)       => "picture",
      metadata::Data::Unknown(_)       => "unknown",
    });
  }
}
