use anyhow::{Context, Result, anyhow};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub mod types;

const HEADER_LEN: usize = 4;
const MAX_FRAME_LEN: usize = 8192;

pub trait FrameData: Sized {
    fn from_data(id: u16, data: &[u8]) -> Result<Self, bitcode::Error>;
    fn to_data(&self) -> (u16, Vec<u8>);
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

    fn parse_frame<F: FrameData>(&mut self) -> Result<Option<F>> {
        // If there aren't even enough bytes for the header, return immediately
        if self.buf.len() < HEADER_LEN {
            return Ok(None);
        }

        let id = u16::from_be_bytes(self.buf[0..2].try_into().unwrap());
        let frame_len = u16::from_be_bytes(self.buf[2..4].try_into().unwrap()) as usize;

        if frame_len > MAX_FRAME_LEN {
            return Err(anyhow!(
                "frame of len {frame_len} is greater than max frame len {MAX_FRAME_LEN}"
            ));
        }

        // If there aren't enough bytes to contain both the header and frame data, return
        // (but make sure enough space is reserved for incoming data)
        if self.buf.len() < HEADER_LEN + frame_len {
            self.buf.reserve(HEADER_LEN + frame_len - self.buf.len());
            return Ok(None);
        }

        // Get frame data specifically as a vec
        let data = &self.buf[HEADER_LEN..HEADER_LEN + frame_len];

        let frame = F::from_data(id, data).context("failed to decode frame data")?;

        // Remove header and frame data from buffer
        self.buf.drain(..HEADER_LEN + frame_len);

        Ok(Some(frame))
    }

    pub async fn read_frame<F: FrameData>(&mut self) -> Result<Option<F>> {
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
                    return Err(anyhow!("connection reset by peer"));
                }
            }
        }
    }

    pub async fn write_frame<F: FrameData>(&mut self, frame: F) -> Result<()> {
        let (id, mut data) = frame.to_data();

        if data.len() > MAX_FRAME_LEN {
            return Err(anyhow!(
                "frame of len {} is greater than max frame len {MAX_FRAME_LEN}",
                data.len()
            ));
        }

        let mut write_buf = Vec::with_capacity(HEADER_LEN + data.len());

        write_buf.extend_from_slice(&id.to_be_bytes());
        write_buf.extend_from_slice(&(data.len() as u16).to_be_bytes());
        write_buf.append(&mut data);

        self.stream
            .write_all(&write_buf)
            .await
            .context("failed to write frame to socket")
    }
}
