use metadata;
use frame;
use subframe;

use metadata::{Metadata, StreamInfo};
use frame::frame_parser;
use utility::{
  ErrorKind, ByteStream, ReadStream, Sample, SampleSize, StreamProducer,
  many_metadata,
};

use std::io;
use std::usize;
use std::fs::File;

/// FLAC stream that decodes and hold file information.
pub struct Stream<P: StreamProducer> {
  info: StreamInfo,
  metadata: Vec<Metadata>,
  producer: P,
}

/// Alias for a FLAC stream produced from `Read`.
pub type StreamReader<R>  = Stream<ReadStream<R>>;

/// Alias for a FLAC stream produced from a byte stream buffer.
pub type StreamBuffer<'a> = Stream<ByteStream<'a>>;

impl<P> Stream<P> where P: StreamProducer {
  /// Constructor for the default state of a FLAC stream.
  #[inline]
  pub fn new<R: io::Read>(reader: R) -> Result<StreamReader<R>, ErrorKind> {
    let producer = ReadStream::new(reader);

    Stream::from_stream_producer(producer)
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
  /// * `ErrorKind::IO(io::ErrorKind::NotFound)` is returned when the given
  ///   filename isn't found.
  /// * `ErrorKind::IO(io::ErrorKind::InvalidData)` is returned when the data
  ///   within the file isn't valid FLAC data.
  /// * Several different parser specific errors that are structured as
  ///   `ErrorKind::<parser_name>Parser`.
  /// * Several different invalidation specific errors that are
  ///   structured as `ErrorKind::Invalid<invalidation_name>`.
  #[inline]
  pub fn from_file(filename: &str) -> Result<StreamReader<File>, ErrorKind> {
    File::open(filename).map_err(|e| ErrorKind::IO(e.kind()))
                        .and_then(|file| {
      let producer = ReadStream::new(file);

      Stream::from_stream_producer(producer)
    })
  }

  /// Constructs a decoder with the given buffer.
  ///
  /// This constructor assumes that an entire FLAC file is in the buffer.
  ///
  /// # Failures
  ///
  /// * `ErrorKind::IO(io::ErrorKind::InvalidData)` is returned when the data
  ///   within the file isn't valid FLAC data.
  /// * Several different parser specific errors that are structured as
  ///   `ErrorKind::<parser_name>Parser`.
  /// * Several different invalidation specific errors that are
  ///   structured as `ErrorKind::Invalid<invalidation_name>`.
  #[inline]
  pub fn from_buffer(buffer: &[u8]) -> Result<StreamBuffer, ErrorKind> {
    let producer = ByteStream::new(buffer);

    Stream::from_stream_producer(producer)
  }

  fn from_stream_producer(mut producer: P) -> Result<Self, ErrorKind> {
    let mut stream_info = Default::default();
    let mut metadata    = Vec::new();

    many_metadata(&mut producer, |block| {
      if let metadata::Data::StreamInfo(info) = block.data {
        stream_info = info;
      } else {
        metadata.push(block);
      }
    }).map(|_| {
      Stream {
        info: stream_info,
        metadata: metadata,
        producer: producer,
      }
    })
  }

  /// Returns an iterator over the decoded samples.
  #[inline]
  pub fn iter<S: SampleSize>(&mut self) -> Iter<P, S::Extended> {
    let samples_left = self.info.total_samples;
    let channels     = self.info.channels as usize;
    let block_size   = self.info.max_block_size as usize;
    let buffer_size  = block_size * channels;

    Iter {
      stream: self,
      channel: 0,
      block_size: 0,
      sample_index: 0,
      samples_left: samples_left,
      buffer: vec![S::Extended::from_i8(0); buffer_size]
    }
  }

  fn next_frame<S>(&mut self, buffer: &mut [S]) -> Option<usize>
   where S: Sample {
    let stream_info = &self.info;

    loop {
      match self.producer.parse(|i| frame_parser(i, stream_info, buffer)) {
        Ok(frame)                => {
          let channels   = frame.header.channels as usize;
          let block_size = frame.header.block_size as usize;
          let subframes  = frame.subframes[0..channels].iter();

          for (channel, subframe) in subframes.enumerate() {
            let start  = channel * block_size;
            let end    = (channel + 1) * block_size;
            let output = &mut buffer[start..end];

            subframe::decode(&subframe, block_size, output);
          }

          frame::decode(frame.header.channel_assignment, buffer);

          return Some(block_size);
        }
        Err(ErrorKind::Continue) => continue,
        Err(_)                   => return None,
      }
    }
  }
}

/// An iterator over a reference of the decoded FLAC stream.
pub struct Iter<'a, P, S>
 where P: 'a + StreamProducer,
       S: Sample{
  stream: &'a mut Stream<P>,
  channel: usize,
  block_size: usize,
  sample_index: usize,
  samples_left: u64,
  buffer: Vec<S>,
}

impl<'a, P, S> Iterator for Iter<'a, P, S>
 where P: StreamProducer,
       S: Sample {
  type Item = S::Normal;

  fn next(&mut self) -> Option<Self::Item> {
    if self.sample_index == self.block_size {
      let buffer = &mut self.buffer;

      if let Some(block_size) = self.stream.next_frame(buffer) {
        self.sample_index = 0;
        self.block_size   = block_size;
      } else {
        return None;
      }
    }

    let channels = self.stream.info.channels as usize;
    let index    = self.sample_index + (self.channel * self.block_size);
    let sample   = unsafe { *self.buffer.get_unchecked(index) };

    self.channel += 1;

    // Reset current channel
    if self.channel == channels {
      self.channel       = 0;
      self.sample_index += 1;
      self.samples_left -= 1;
    }

    S::to_normal(sample)
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

//impl<'a, P, S> IntoIterator for &'a mut Stream<P>
// where P: StreamProducer,
//       S: Sample {
//  type Item     = S::Normal;
//  type IntoIter = Iter<'a, P, S>;
//
//  #[inline]
//  fn into_iter(self) -> Self::IntoIter {
//    self.iter()
//  }
//}
