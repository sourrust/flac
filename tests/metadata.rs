extern crate flac;

use flac::metadata;
use std::io::ErrorKind;

#[test]
fn test_get_stream_info() {
  let result = metadata::get_stream_info("tests/assets/input-SCPAP.flac");

  assert!(result.is_ok());
}

#[test]
fn test_get_vorbis_comment() {
  let tags    = metadata::get_vorbis_comment("tests/assets/input-SVAUP.flac");
  let no_tags = metadata::get_vorbis_comment("tests/assets/input-SCPAP.flac");

  assert!(tags.is_ok(), "Should have vorbis comments");
  assert_eq!(no_tags.unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn test_get_cue_sheet() {
  let cue_sheet    = metadata::get_cue_sheet("tests/assets/input-SCPAP.flac");
  let no_cue_sheet = metadata::get_cue_sheet("tests/assets/input-SVAUP.flac");

  assert!(cue_sheet.is_ok(), "Should have a cue sheet");
  assert_eq!(no_cue_sheet.unwrap_err().kind(), ErrorKind::NotFound);
}
