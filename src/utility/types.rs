use nom::IResult;

pub trait StreamProducer {
  fn parse<F, T>(&mut self, f: F) -> Result<T, ErrorKind>
   where F: FnOnce(&[u8]) -> IResult<&[u8], T>;
}
