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
