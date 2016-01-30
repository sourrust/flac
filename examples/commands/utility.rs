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
