use std::io;

use bytes::BytesMut;
use tokio_util::codec::Decoder;

use persist_core::error::Error;

#[derive(Debug, Clone, Default)]
pub struct LogDecoder {
    next_index: usize,
}

impl LogDecoder {
    pub fn new() -> Self {
        Self { next_index: 0 }
    }
}

impl Decoder for LogDecoder {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut iter = src[self.next_index..].iter().enumerate().peekable();

        let position = loop {
            let next = iter.next();
            match next {
                Some((index, b'\n')) => break Some(self.next_index + index),
                // Special logic to not split twice when encountering a CRLF ("\r\n").
                // We always wait for an additional character after a CR ('\r') and we make sure it isn't a LF ('\n').
                Some((index, b'\r')) => match iter.peek() {
                    Some((index, b'\n')) => break Some(self.next_index + index),
                    None => break None,
                    _ => break Some(self.next_index + index),
                },
                Some(_) => {}
                None => {
                    self.next_index = src.len();
                    break None;
                }
            }
        };

        position
            .map(|index| {
                self.next_index = 0;
                let line = src.split_to(index + 1);
                let line = &line[..line.len() - 1];
                let line = std::str::from_utf8(line).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Unable to decode input as UTF8")
                })?;
                Ok(line.to_string())
            })
            .transpose()
    }

    fn decode_eof(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.next_index = 0;

        if src.is_empty() {
            Ok(None)
        } else {
            let line = src.split();
            let line = if matches!(line.last(), Some(b'\r')) {
                &line[..line.len() - 1]
            } else {
                &line[..]
            };
            let line = std::str::from_utf8(line).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Unable to decode input as UTF8")
            })?;
            Ok(Some(line.to_string()))
        }
    }
}
