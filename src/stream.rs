use nom;
use nom::{Err, IResult};

use metadata;
use frame;
use subframe;

use metadata::{Metadata, StreamInfo, metadata_parser};
use frame::{frame_parser, Frame};
use utility::{ErrorKind, ByteStream, ReadStream, StreamProducer};

use std::io;
use std::usize;
use std::fs::File;

enum ParserState {
  Marker,
  Metadata,
  Frame,
}

/// FLAC stream that decodes and hold file information.
pub struct Stream<P: StreamProducer> {
  info: StreamInfo,
  metadata: Vec<Metadata>,
  frames: Vec<Frame>,
  state: ParserState,
  output: Vec<i32>,
  frame_index: usize,
  producer: P,
}

fn parser<'a>(input: &'a [u8], is_start: &mut bool)
              -> IResult<&'a [u8], Metadata> {
  let mut slice = input;

  if *is_start {
    let (i, _) = try_parse!(slice, tag!("fLaC"));

    slice     = i;
    *is_start = false;
  }

  metadata_parser(slice)
}

impl<P> Stream<P> where P: StreamProducer {
  /// Constructor for the default state of a FLAC stream.
  ///
  /// This doesn't actually decode anything, it just hold the default values
  /// of each field.
  pub fn new() -> Stream {
    Stream {
      info: StreamInfo::new(),
      metadata: Vec::new(),
      frames: Vec::new(),
      state: ParserState::Marker,
      output: Vec::new(),
      frame_index: 0,
    }
  }

  /// Returns information for the current stream.
  pub fn info(&self) -> StreamInfo {
    self.info
  }

  /// Returns a slice of `Metadata`
  ///
  /// This slice excludes `StreamInfo`, which is located in `Stream::info`.
  /// Everything else is related to metadata for the FLAC stream is in the
  /// slice.
  pub fn metadata(&self) -> &[Metadata] {
    &self.metadata
  }

  /// Constructs a decoder with the given file name.
  ///
  /// # Failures
  ///
  /// * `ErrorKind::NotFound` is returned when the given filename isn't found.
  /// * `ErrorKind::InvalidData` is returned when the data within the file
  ///   isn't valid FLAC data.
  pub fn from_file(filename: &str) -> io::Result<Stream<ReadStream<File>>> {
    File::open(filename).and_then(|file| {
      let producer  = ReadStream::new(file);
      let error_str = format!("parser: couldn't parse the given file {}",
                              filename);

      Stream::from_stream_producer(producer, &error_str)
    })
  }

  /// Constructs a decoder with the given buffer.
  ///
  /// This constructor assumes that an entire FLAC file is in the buffer.
  ///
  /// # Failures
  ///
  /// * `ErrorKind::InvalidData` is returned when the data within the buffer
  ///   isn't valid FLAC data.
  pub fn from_buffer(buffer: &[u8]) -> io::Result<Stream> {
    let mut producer = ByteStream::new(buffer);
    let error_str    = "parser: couldn't parse the buffer";

    Stream::from_stream_producer(&mut producer, error_str)
  }

  fn from_stream_producer(mut producer: P, error_str: &str)
                          -> io::Result<Self> {
    let mut is_start    = true;
    let mut is_error    = false;
    let mut stream_info = StreamInfo::new();
    let mut metadata    = Vec::new();

    loop {
      match producer.parse(|i| parser(i, &mut is_start)) {
        Ok(block)                  => {
          let is_last = block.is_last;

          if let metadata::Data::StreamInfo(info) = block.data {
            stream_info = info;
          } else {
            metadata.push(block);
          }

          if is_last {
            break;
          }
        }
        Err(ErrorKind::EndOfInput) => break,
        Err(ErrorKind::Continue)   => continue,
        Err(_)                     => {
          is_error = true;

          break;
        }
      }
    }

    if !is_error {
      let channels    = stream_info.channels as usize;
      let block_size  = stream_info.max_block_size as usize;
      let output_size = block_size * channels;
      let mut output  = Vec::with_capacity(output_size);

      unsafe { output.set_len(output_size) }

      Ok(Stream {
        info: stream_info,
        metadata: metadata,
        frames: Vec::new(),
        state: ParserState::Marker,
        output: output,
        frame_index: 0,
        producer: producer,
      })
    } else {
      Err(io::Error::new(io::ErrorKind::InvalidData, error_str))
    }
  }

  /// Returns an iterator over the decoded samples.
  pub fn iter(&mut self) -> Iter<P> {
    let samples_left = self.info.total_samples;

    Iter {
      stream: self,
      channel: 0,
      block_size: 0,
      sample_index: 0,
      samples_left: samples_left,
    }
  }

  fn next_frame<'a>(&'a mut self) -> Option<&'a [i32]> {
    if self.frames.is_empty() || self.frame_index >= self.frames.len() {
      None
    } else {
      let frame       = &self.frames[self.frame_index];
      let channels    = frame.header.channels as usize;
      let block_size  = frame.header.block_size as usize;
      let mut channel = 0;

      for subframe in &frame.subframes[0..channels] {
        let start  = channel * block_size;
        let end    = (channel + 1) * block_size;
        let output = &mut self.output[start..end];

        subframe::decode(&subframe, output);

        channel += 1;
      }

      frame::decode(frame.header.channel_assignment, &mut self.output);

      self.frame_index += 1;

      Some(&self.output[0..(block_size * channels)])
    }
  }

  fn handle_marker<'a>(&mut self, input: &'a [u8]) -> IResult<&'a [u8], ()> {
    let kind = nom::ErrorKind::Custom(0);

    match tag!(input, "fLaC") {
      IResult::Done(i, _)    => {
        self.state = ParserState::Metadata;

        IResult::Error(Err::Position(kind, i))
      }
      IResult::Error(_)      => IResult::Error(Err::Code(kind)),
      IResult::Incomplete(n) => IResult::Incomplete(n),
    }
  }

  fn handle_metadata<'a>(&mut self, input: &'a [u8])
                         -> IResult<&'a [u8], ()> {
    let kind = nom::ErrorKind::Custom(1);

    match metadata_parser(input) {
      IResult::Done(i, block) => {
        let is_last = block.is_last;

        if let metadata::Data::StreamInfo(info) = block.data {
          self.info = info;
        } else {
          self.metadata.push(block);
        }

        if is_last {
          self.state = ParserState::Frame;
        }

        IResult::Error(Err::Position(kind, i))
      }
      IResult::Error(_)      => IResult::Error(Err::Code(kind)),
      IResult::Incomplete(n) => IResult::Incomplete(n),
    }
  }

  fn handle_frame<'a>(&mut self, input: &'a [u8]) -> IResult<&'a [u8], ()> {
    let kind = nom::ErrorKind::Custom(2);

    match frame_parser(input, &self.info) {
      IResult::Done(i, frame) => {
        self.frames.push(frame);

        IResult::Error(Err::Position(kind, i))
      }
      IResult::Error(_)      => IResult::Error(Err::Code(kind)),
      IResult::Incomplete(n) => IResult::Incomplete(n),
    }
  }

  fn handle<S: StreamProducer>(&mut self, stream: &mut S)
                               -> Result<(), ErrorKind> {
    stream.parse(|input| {
      match self.state {
        ParserState::Marker   => self.handle_marker(input),
        ParserState::Metadata => self.handle_metadata(input),
        ParserState::Frame    => self.handle_frame(input),
      }
    })
  }
}

/// An iterator over a reference of the decoded FLAC stream.
pub struct Iter<'a, P> where P: 'a + StreamProducer {
  stream: &'a mut Stream<P>,
  channel: usize,
  block_size: usize,
  sample_index: usize,
  samples_left: u64,
}

impl<'a, P> Iterator for Iter<'a, P> where P: StreamProducer {
  type Item = i32;

  fn next(&mut self) -> Option<Self::Item> {
    if self.sample_index == self.block_size {
      let frame_index = self.stream.frame_index;

      if self.stream.next_frame().is_none() {
        return None;
      } else {
        let frame = &self.stream.frames[frame_index];

        self.sample_index = 0;
        self.block_size   = frame.header.block_size as usize;
      }
    }

    let channels = self.stream.info.channels as usize;
    let index    = self.sample_index + (self.channel * self.block_size);
    let sample   = self.stream.output[index];

    self.channel += 1;

    // Reset current channel
    if self.channel == channels {
      self.channel       = 0;
      self.sample_index += 1;
      self.samples_left -= 1;
    }

    Some(sample)
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let samples_left = self.samples_left as usize;
    let max_value    = usize::max_value() as u64;

    // There is a chance that samples_left will be larger than a usize since
    // it is a u64. Make the upper bound None when it is.
    if self.samples_left > max_value {
      (samples_left, None)
    } else {
      (samples_left, Some(samples_left))
    }
  }
}

impl<'a, P> IntoIterator for &'a mut Stream<P>
 where P: StreamProducer {
  type Item     = i32;
  type IntoIter = Iter<'a, P>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}
