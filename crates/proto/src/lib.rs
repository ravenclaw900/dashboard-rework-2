use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub mod types;

const HEADER_LEN: usize = 4;
const MAX_FRAME_LEN: usize = 8192;

pub struct Frame {
    id: u16,
    data: Vec<u8>,
}

impl Frame {
    pub fn from_encode<T: bitcode::Encode>(id: u16, val: &T) -> Self {
        let data = bitcode::encode(val);

        Self { id, data }
    }

    pub fn into_decode<T: bitcode::DecodeOwned>(self) -> Result<(u16, T), bitcode::Error> {
        let val = bitcode::decode(&self.data)?;

        Ok((self.id, val))
    }

    pub fn into_data(self) -> (u16, Vec<u8>) {
        (self.id, self.data)
    }
}

pub struct DashboardSocket {
    stream: TcpStream,
    buf: Vec<u8>,
}

impl DashboardSocket {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buf: Vec::with_capacity(1024),
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, io::Error> {
        // If there aren't even enough bytes for the header, return immediately
        if self.buf.len() < HEADER_LEN {
            return Ok(None);
        }

        let id = u16::from_be_bytes(self.buf[0..2].try_into().unwrap());
        let frame_len = u16::from_be_bytes(self.buf[2..4].try_into().unwrap()) as usize;

        if frame_len > MAX_FRAME_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "frame of len {} is greater than max frame len {}",
                    frame_len, MAX_FRAME_LEN
                ),
            ));
        }

        // If there aren't enough bytes to contain both the header and frame data, return
        // (but make sure enough space is reserved for incoming data)
        if self.buf.len() < HEADER_LEN + frame_len {
            self.buf.reserve(HEADER_LEN + frame_len - self.buf.len());
            return Ok(None);
        }

        // Get frame data specifically as a vec
        let data = self.buf[HEADER_LEN..HEADER_LEN + frame_len].to_vec();

        // Remove header and frame data from buffer
        self.buf.drain(..HEADER_LEN + frame_len);

        Ok(Some(Frame { id, data }))
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, io::Error> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            let n = self.stream.read_buf(&mut self.buf).await?;

            if n == 0 {
                // If buffer is empty, we weren't in the middle of receiving anything
                // Otherwise, something definitely went wrong on the other end
                if self.buf.is_empty() {
                    return Ok(None);
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionReset,
                        "connection reset by peer",
                    ));
                }
            }
        }
    }

    pub async fn write_frame(&mut self, mut frame: Frame) -> Result<(), io::Error> {
        if frame.data.len() > MAX_FRAME_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "frame of len {} is greater than max frame len {}",
                    frame.data.len(),
                    MAX_FRAME_LEN
                ),
            ));
        }

        let mut write_buf = Vec::with_capacity(HEADER_LEN + frame.data.len());

        write_buf.extend_from_slice(&frame.id.to_be_bytes());
        write_buf.extend_from_slice(&(frame.data.len() as u16).to_be_bytes());
        write_buf.append(&mut frame.data);

        self.stream.write_all(&write_buf).await
    }
}
