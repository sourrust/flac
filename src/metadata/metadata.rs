use std::io;
use std::u32;
use std::fs::File;

use utility::{ErrorKind, ReadStream, many_metadata};

use metadata::{
  Metadata, Data,
  StreamInfo, CueSheet, VorbisComment, Picture,
  PictureType,
};

// Will return true when the unwrapped value of `option` and `other` match
// or `option` is `Option::None`, otherwise false.
#[inline]
pub fn optional_eq<T: Eq>(option: Option<T>, other: T) -> bool {
  option.map_or(true, |value| value == other)
}

// With the given filename, return all metadata blocks available.
//
// This function expects a flac file, but will return a proper `Result::Err`
// when things go wrong.
//
// # Failures
//
// * `ErrorKind::IO(io::ErrorKind::NotFound)` is returned when the given
//   filename isn't found.
// * `ErrorKind::IO(io::ErrorKind::InvalidData)` is returned when the data
//   within the file isn't valid FLAC data.
// * Several different parser specific errors that are structured as
//   `ErrorKind::<parser_name>Parser`.
pub fn get_metadata(filename: &str) -> Result<Vec<Metadata>, ErrorKind> {
  File::open(filename).map_err(|e| ErrorKind::IO(e.kind()))
                      .and_then(|file| {
    let mut stream   = ReadStream::new(file);
    let mut metadata = Vec::new();

    let result = many_metadata(&mut stream, |block| metadata.push(block));

    if let Err(kind) = result {
      match kind {
        ErrorKind::HeaderParser |
        ErrorKind::Unknown      => Err(ErrorKind::IO(
                                     io::ErrorKind::InvalidData)),
        _                       => Err(kind),
      }
    } else {
      Ok(metadata)
    }
  })
}

/// Reads and returns the `StreamInfo` metadata block of the given FLAC
/// file.
///
/// # Failures
///
/// * `ErrorKind::NotFound` is returned when the there is no `StreamInfo`
///   within the file.
/// * `ErrorKind::IO(io::ErrorKind::NotFound)` is returned when the given
///   filename isn't found.
/// * `ErrorKind::IO(io::ErrorKind::InvalidData)` is returned when the data
///   within the file isn't valid FLAC data.
/// * Several different parser specific errors that are structured as
///   `ErrorKind::<parser_name>Parser`.
///
/// # Examples
///
/// Handling errors might look something like this:
///
/// ```
/// use flac::metadata;
///
/// match metadata::get_stream_info("path/to/file.flac") {
///   Ok(stream_info) => {
///     // Use the stream_info variable...
///   }
///   Err(error)      => println!("{:?}", error),
/// }
/// ```
///
/// Or just ignore the errors:
///
/// ```no_run
/// use flac::metadata;
///
/// let stream_info = metadata::get_stream_info("path/to/file.flac").unwrap();
/// ```
pub fn get_stream_info(filename: &str) -> Result<StreamInfo, ErrorKind> {
  get_metadata(filename).and_then(|blocks| {
    let mut result = Err(ErrorKind::NotFound);

    for block in blocks {
      if let Data::StreamInfo(stream_info) = block.data {
        result = Ok(stream_info);
        break;
      }
    }

    result
  })
}

/// Reads and returns the `VorbisComment` metadata block of the given FLAC
/// file.
///
/// # Failures
///
/// * `ErrorKind::NotFound` is returned when the there is no `VorbisComment`
///   within the file.
/// * `ErrorKind::IO(io::ErrorKind::NotFound)` is returned when the given
///   filename isn't found.
/// * `ErrorKind::IO(io::ErrorKind::InvalidData)` is returned when the data
///   within the file isn't valid FLAC data.
/// * Several different parser specific errors that are structured as
///   `ErrorKind::<parser_name>Parser`.
///
/// # Examples
///
/// Handling errors might look something like this:
///
/// ```
/// use flac::metadata;
///
/// match metadata::get_vorbis_comment("path/to/file.flac") {
///   Ok(vorbis_comment) => {
///     // Use the vorbis_comment variable...
///   }
///   Err(error)         => println!("{:?}", error),
/// }
/// ```
///
/// Or just ignore the errors:
///
/// ```no_run
/// use flac::metadata;
///
/// let vorbis_comment =
///   metadata::get_vorbis_comment("path/to/file.flac").unwrap();
/// ```
pub fn get_vorbis_comment(filename: &str)
                          -> Result<VorbisComment, ErrorKind> {
  get_metadata(filename).and_then(|blocks| {
    let mut result = Err(ErrorKind::NotFound);

    for block in blocks {
      if let Data::VorbisComment(vorbis_comment) = block.data {
        result = Ok(vorbis_comment);
        break;
      }
    }

    result
  })
}

/// Reads and returns the `CueSheet` metadata block of the given FLAC file.
///
/// # Failures
///
/// * `ErrorKind::NotFound` is returned when the there is no `CueSheet`
///   within the file.
/// * `ErrorKind::IO(io::ErrorKind::NotFound)` is returned when the given
///   filename isn't found.
/// * `ErrorKind::IO(io::ErrorKind::InvalidData)` is returned when the data
///   within the file isn't valid FLAC data.
/// * Several different parser specific errors that are structured as
///   `ErrorKind::<parser_name>Parser`.
///
/// # Examples
///
/// Handling errors might look something like this:
///
/// ```
/// use flac::metadata;
///
/// match metadata::get_cue_sheet("path/to/file.flac") {
///   Ok(cue_sheet) => {
///     // Use the cue_sheet variable...
///   }
///   Err(error)    => println!("{:?}", error),
/// }
/// ```
///
/// Or just ignore the errors
///
/// ```no_run
/// use flac::metadata;
///
/// let cue_sheet = metadata::get_cue_sheet("path/to/file.flac").unwrap();
/// ```
pub fn get_cue_sheet(filename: &str) -> Result<CueSheet, ErrorKind> {
  get_metadata(filename).and_then(|blocks| {
    let mut result = Err(ErrorKind::NotFound);

    for block in blocks {
      if let Data::CueSheet(cue_sheet) = block.data {
        result = Ok(cue_sheet);
        break;
      }
    }

    result
  })
}

/// Reads and returns a `Picture` metadata block of the given FLAC file.
///
/// There can be more than one `Picture` block in a file and this function
/// takes optional, that being `Option<T>`, parameters that act as
/// constraints to search within. The `Picture` with the largest area
/// matching all constraints will be returned.
///
/// Putting `None` into any of the optional constraints conveys that you
/// want any of that parameter. Otherwise it will try to look for the image
/// that matches within the given constraints.
///
/// # Failures
///
/// * `ErrorKind::NotFound` is returned when the there is no `Picture`
///   within the file, or no `Picture` that fits the given constraints.
/// * `ErrorKind::IO(io::ErrorKind::NotFound)` is returned when the given
///   filename isn't found.
/// * `ErrorKind::IO(io::ErrorKind::InvalidData)` is returned when the data
///   within the file isn't valid FLAC data.
/// * Several different parser specific errors that are structured as
///   `ErrorKind::<parser_name>Parser`.
///
/// # Examples
///
/// Handling errors might look something like this:
///
/// ```
/// use flac::metadata;
/// use flac::metadata::PictureType;
///
/// let result = metadata::get_picture("path/to/file.flac",
///                                    Some(PictureType::FileIcon),
///                                    Some("image/gif"),
///                                    None, None, None,
///                                    None, None);
///
/// match result {
///   Ok(picture) => {
///     // Use the picture variable...
///   }
///   Err(error)  => println!("{:?}", error),
/// }
/// ```
///
/// Or just ignore the errors:
///
/// ```no_run
/// use flac::metadata;
/// use flac::metadata::PictureType;
///
/// let picture = metadata::get_picture("path/to/file.flac",
///                                     Some(PictureType::FileIcon),
///                                     Some("image/gif"),
///                                     None, None, None,
///                                     None, None).unwrap();
/// ```
pub fn get_picture(filename: &str,
                   picture_type: Option<PictureType>,
                   mime_type: Option<&str>,
                   description: Option<&str>,
                   max_width: Option<u32>,
                   max_height: Option<u32>,
                   max_depth: Option<u32>,
                   max_colors: Option<u32>)
                   -> Result<Picture, ErrorKind> {
  get_metadata(filename).and_then(|blocks| {
    let mut result = Err(ErrorKind::NotFound);

    let mut max_area_seen  = 0;
    let mut max_depth_seen = 0;

    let max_value      = u32::max_value();
    let max_width_num  = max_width.unwrap_or(max_value);
    let max_height_num = max_height.unwrap_or(max_value);
    let max_depth_num  = max_depth.unwrap_or(max_value);
    let max_colors_num = max_colors.unwrap_or(max_value);

    for block in blocks {
      if let Data::Picture(picture) = block.data {
        let area = (picture.width as u64) * (picture.height as u64);

        if optional_eq(picture_type, picture.picture_type) &&
           optional_eq(mime_type, &picture.mime_type) &&
           optional_eq(description, &picture.description) &&
           picture.width <= max_width_num &&
           picture.height <= max_height_num &&
           picture.depth <= max_depth_num &&
           picture.colors <= max_colors_num &&
           (area > max_area_seen || (area == max_area_seen &&
                                     picture.depth > max_depth_seen)) {
          max_area_seen  = area;
          max_depth_seen = picture.depth;
          result         = Ok(picture);
        }
      }
    }

    result
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io;

  use utility::ErrorKind;

  #[test]
  #[should_panic]
  fn test_panic_optional_eq() {
    assert!(optional_eq(Some(1), 0));
  }

  #[test]
  fn test_optional_eq() {
    assert!(optional_eq(None, 0), "Should always return true when None");
    assert!(optional_eq(Some(0), 0), "Should return true (Some(0) == 0)");
  }

  #[test]
  fn test_get_metadata() {
    let not_found    = get_metadata("non-existent/file.txt");
    let invalid_data = get_metadata("README.md");
    let result       = get_metadata("tests/assets/input-SVAUP.flac");

    assert_eq!(not_found.unwrap_err(), ErrorKind::IO(
                                         io::ErrorKind::NotFound));
    assert_eq!(invalid_data.unwrap_err(), ErrorKind::IO(
                                            io::ErrorKind::InvalidData));
    assert!(result.is_ok());
  }
}
