pub const USAGE: &'static str = "
Usage: metadata comments [options] <filename>
       metadata comments --help

Options:
  --vendor       Show the vendor string.
  --name=NAME    Show the comments matching the `NAME`.
  --export=FILE  Export to file.
  -h, --help     Show this message.
";

#[derive(Debug, RustcDecodable)]
pub struct Arguments {
  arg_filename: String,
  flag_vendor: bool,
  flag_name: Option<String>,
  flag_export: Option<String>,
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
