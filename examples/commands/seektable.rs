pub const USAGE: &'static str = "
Usage: metadata seektable <filename>
       metadata seektable --help

Options:
  -h, --help  Show this message.
";

#[derive(Debug, RustcDecodable)]
pub struct Arguments {
  arg_filename: String,
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
