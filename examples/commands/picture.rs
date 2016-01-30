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
