extern crate docopt;
extern crate flac;
extern crate rustc_serialize;

#[macro_use]
mod commands;

use std::env;

use commands::{streaminfo, comments, seektable, picture};
use docopt::Docopt;

const USAGE: &'static str = "
Usage: metadata <command> [<args>...]
       metadata [options]

Options:
  -h, --help  Show this message.

Commands:
  streaminfo  Display stream information.
  comments    Display or export comment tags.
  seektable   Display seek table.
  picture     Export pictures.
";

#[derive(Debug, RustcDecodable)]
struct Arguments {
  arg_command: Option<Command>,
  arg_args: Vec<String>,
}

#[derive(Clone, Copy, Debug, RustcDecodable)]
enum Command {
  StreamInfo,
  Comments,
  SeekTable,
  Picture,
}

fn handle_subcommand(command: Command) {
  match command {
    Command::StreamInfo => command!(streaminfo),
    Command::Comments   => command!(comments),
    Command::SeekTable  => command!(seektable),
    Command::Picture    => command!(picture),
  }
}

fn main() {
  let args: Arguments = Docopt::new(USAGE)
    .and_then(|d| d.options_first(true).decode())
    .unwrap_or_else(|e| e.exit());

  if let Some(command) = args.arg_command {
    handle_subcommand(command);
  } else {
    println!("{}", USAGE);
  }
}
