use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use proto::{
    DashboardSocket, Frame,
    types::{Handshake, Request},
};
use sysinfo::System;
use tokio::{net::TcpStream, sync::mpsc};

use crate::getters;

macro_rules! call_getter {
    ($id:expr, blocking $getter:expr) => {{
        // Should only return an error here if the getter panics, which we're assuming never happens
        let getter_res = tokio::task::spawn_blocking(move || $getter).await.unwrap();
        Frame::from_encode($id, &getter_res)
    }};
}

pub type SharedSystem = Arc<Mutex<System>>;

pub struct BackendClient {
    socket: DashboardSocket,
    system: SharedSystem,
    // config: &'static BackendConfig,
}

impl BackendClient {
    pub async fn new(addr: SocketAddr, system: SharedSystem) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .context("failed to connect to frontend")?;

        Ok(Self {
            socket: DashboardSocket::new(stream),
            system,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        self.send_handshake().await?;

        loop {
            tokio::select! {
                frame_result = self.socket.read_frame() => {
                    let frame = frame_result
                        .context("failed to read frame from frontend")?
                        .context("frontend unexpectedly disconnected")?;

                    let (id, req) = frame.into_decode::<Request>().context("failed to decode request")?;

                    let handler = RequestHandler::new(id, tx.clone(), req, self.system.clone());
                    tokio::spawn(handler.run());
                }
                chan_result = rx.recv() => {
                    // Since we hold a copy of the sender, it should be impossible for this to return None
                    let frame = chan_result.unwrap();

                    self.socket.write_frame(frame).await.context("failed to send response")?;
                }
            }
        }
    }

    async fn send_handshake(&mut self) -> Result<()> {
        let handshake = Handshake {
            nickname: "test".to_string(),
        };

        self.socket
            .write_frame(Frame::from_encode(0, &handshake))
            .await
            .context("failed to send handshake")
    }
}

struct RequestHandler {
    id: u16,
    tx: mpsc::UnboundedSender<Frame>,
    req: Request,
    system: SharedSystem,
}

impl RequestHandler {
    fn new(id: u16, tx: mpsc::UnboundedSender<Frame>, req: Request, system: SharedSystem) -> Self {
        Self {
            id,
            tx,
            req,
            system,
        }
    }

    async fn run(self) {
        let frame = match self.req {
            Request::System => {
                call_getter!(
                    self.id,
                    blocking getters::system(&mut self.system.lock().unwrap())
                )
            }
        };

        let _ = self.tx.send(frame);
    }
}
