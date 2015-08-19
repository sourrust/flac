extern crate flac;

use flac::metadata;
use flac::metadata::{Picture, PictureType};
use std::io::ErrorKind;

fn compare_all_but_data(picture: &Picture, other_picture: &Picture) -> bool {
  (picture.picture_type == other_picture.picture_type) &&
    (picture.mime_type == other_picture.mime_type) &&
    (picture.description == other_picture.description) &&
    (picture.width == other_picture.width) &&
    (picture.height == other_picture.height) &&
    (picture.depth == other_picture.depth) &&
    (picture.colors == other_picture.colors)
}

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

#[test]
fn test_get_picture() {
  let picture_file = "tests/assets/input-pictures.flac";
  let all_options  = metadata::get_picture(picture_file,
                                           Some(PictureType::Other),
                                           Some("image/gif"),
                                           Some(""),
                                           Some(16),
                                           Some(14),
                                           Some(24),
                                           Some(128));
  let picture1 = Picture {
    picture_type: PictureType::Other,
    mime_type: "image/gif".to_owned(),
    description: "".to_owned(),
    width: 16,
    height: 14,
    depth: 24,
    colors: 128,
    data: vec![],
  };

  let one_option = metadata::get_picture(picture_file,
                                         Some(PictureType::FileIconStandard),
                                         None, None, None, None, None, None);
  let picture2 = Picture {
    picture_type: PictureType::FileIconStandard,
    mime_type: "image/png".to_owned(),
    description: "8.png".to_owned(),
    width: 32,
    height: 32,
    depth: 32,
    colors: 0,
    data: vec![],
  };

  let no_option = metadata::get_picture(picture_file, None, None, None, None,
                                        None, None, None);
  let picture3  = Picture {
    picture_type: PictureType::FrontCover,
    mime_type: "image/png".to_owned(),
    description: "7.png".to_owned(),
    width: 31,
    height: 47,
    depth: 24,
    colors: 23,
    data: vec![],
  };
  let no_picture = metadata::get_picture("tests/assets/input-SVAUP.flac",
                                         None, None, None, None, None, None,
                                         None);

  assert!(compare_all_but_data(&all_options.unwrap(), &picture1),
          "All constraint options");
  assert!(compare_all_but_data(&one_option.unwrap(), &picture2),
          "One constraint option");
  assert!(compare_all_but_data(&no_option.unwrap(), &picture3),
          "No constraint option");
  assert_eq!(no_picture.unwrap_err().kind(), ErrorKind::NotFound);
}
