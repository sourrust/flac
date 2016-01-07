use nom::Err;

use metadata;
use frame;
use subframe;

use metadata::{Metadata, StreamInfo};
use frame::frame_parser;
use utility::{
  ErrorKind, ByteStream, ReadStream, StreamProducer, many_metadata,
};

use std::io;
use std::usize;
use std::fs::File;

/// FLAC stream that decodes and hold file information.
pub struct Stream<P: StreamProducer> {
  info: StreamInfo,
  metadata: Vec<Metadata>,
  producer: P,
  output: Vec<i32>,
}

impl<P> Stream<P> where P: StreamProducer {
  /// Constructor for the default state of a FLAC stream.
  #[inline]
  pub fn new<R: io::Read>(reader: R) -> io::Result<Stream<ReadStream<R>>> {
    let producer  = ReadStream::new(reader);
    let error_str = "parser: couldn't parse the reader";

    Stream::from_stream_producer(producer, error_str)
  }

  /// Returns information for the current stream.
  #[inline]
  pub fn info(&self) -> StreamInfo {
    self.info
  }

  /// Returns a slice of `Metadata`
  ///
  /// This slice excludes `StreamInfo`, which is located in `Stream::info`.
  /// Everything else is related to metadata for the FLAC stream is in the
  /// slice.
  #[inline]
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
  #[inline]
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
  #[inline]
  pub fn from_buffer(buffer: &[u8]) -> io::Result<Stream<ByteStream>> {
    let producer  = ByteStream::new(buffer);
    let error_str = "parser: couldn't parse the buffer";

    Stream::from_stream_producer(producer, error_str)
  }

  fn from_stream_producer(mut producer: P, error_str: &str)
                          -> io::Result<Self> {
    let mut stream_info = StreamInfo::new();
    let mut metadata    = Vec::new();

    let is_error = many_metadata(&mut producer, |block| {
      if let metadata::Data::StreamInfo(info) = block.data {
        stream_info = info;
      } else {
        metadata.push(block);
      }
    });

    if !is_error {
      Ok(Stream {
        info: stream_info,
        metadata: metadata,
        producer: producer,
        output: Vec::new(),
      })
    } else {
      Err(io::Error::new(io::ErrorKind::InvalidData, error_str))
    }
  }

  /// Returns an iterator over the decoded samples.
  #[inline]
  pub fn iter(&mut self) -> Iter<P> {
    let samples_left = self.info.total_samples;

    if self.output.is_empty() {
      let channels    = self.info.channels as usize;
      let block_size  = self.info.max_block_size as usize;
      let output_size = block_size * channels;

      self.output.reserve_exact(output_size);

      unsafe { self.output.set_len(output_size) }
    }

    Iter {
      stream: self,
      channel: 0,
      block_size: 0,
      sample_index: 0,
      samples_left: samples_left,
    }
  }

  fn next_frame<'a>(&'a mut self) -> Option<usize> {
    let stream_info = self.info();

    loop {
      match self.producer.parse(|i| frame_parser(i, &stream_info)) {
        Ok(frame)                  => {
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

          return Some(block_size);
        }
        Err(ErrorKind::EndOfInput) => return None,
        Err(ErrorKind::Continue)   => continue,
        Err(_)                     => return None,
      }
    }
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
      if let Some(block_size) = self.stream.next_frame() {
        self.sample_index = 0;
        self.block_size   = block_size;
      } else {
        return None;
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

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}
