use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use config::{PROTOCOL_VERSION, backend::BackendConfig};
use proto::{
    DashboardSocket,
    backend::{BackendMessage, Handshake, IdBackendMessage, NoIdBackendMessage},
    frontend::{FrontendMessage, IdFrontendMessage, NoIdFrontendMessage},
};
use sysinfo::{Components, Disks, Networks, System};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{SharedConfig, getters, terminal};

async fn call_blocking_getter<F, R>(ctx: BackendContext, getter: F) -> R
where
    F: FnOnce(BackendContext) -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(|| getter(ctx)).await.unwrap()
}

pub type SharedSystem = Arc<Mutex<SystemComponents>>;

pub struct SystemComponents {
    pub system: System,
    pub components: Components,
    pub disks: Disks,
    pub networks: Networks,
}

impl SystemComponents {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            components: Components::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        }
    }
}

#[derive(Clone)]
pub struct BackendContext {
    pub config: SharedConfig,
    pub system: SharedSystem,
    pub socket_tx: mpsc::UnboundedSender<BackendMessage>,
    pub term_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl BackendContext {
    pub fn system(&mut self) -> impl std::ops::DerefMut<Target = SystemComponents> {
        self.system.lock().unwrap()
    }
}

pub struct BackendClient {
    socket: DashboardSocket,
    context: BackendContext,
    rx: mpsc::UnboundedReceiver<BackendMessage>,
}

impl BackendClient {
    pub async fn new(
        context: BackendContext,
        rx: mpsc::UnboundedReceiver<BackendMessage>,
    ) -> Result<Self> {
        let stream = TcpStream::connect(context.config.frontend_addr)
            .await
            .context("failed to connect to frontend")?;

        Ok(Self {
            socket: DashboardSocket::new(stream),
            context,
            rx,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        self.send_handshake().await?;

        loop {
            tokio::select! {
                frame_result = self.socket.read_frame() => {
                    let req: FrontendMessage = frame_result
                        .context("failed to read frame from frontend")?
                        .context("frontend unexpectedly disconnected")?;

                    let handler = RequestHandler::new(req, self.context.clone());
                    tokio::spawn(handler.run());
                }
                chan_result = self.rx.recv() => {
                    // Since we hold a copy of the sender, it should be impossible for this to return None
                    let frame = chan_result.unwrap();

                    self.socket.write_frame(frame).await.context("failed to send response")?;
                }
            }
        }
    }

    async fn send_handshake(&mut self) -> Result<()> {
        let nickname = self.context.config.nickname.clone();

        let handshake = Handshake {
            nickname,
            version: PROTOCOL_VERSION,
        };

        let msg = NoIdBackendMessage::Handshake(handshake);
        let msg = BackendMessage::NoId(msg);

        self.socket
            .write_frame(msg)
            .await
            .context("failed to send handshake")
    }
}

struct RequestHandler {
    req: FrontendMessage,
    context: BackendContext,
}

impl RequestHandler {
    fn new(req: FrontendMessage, context: BackendContext) -> Self {
        Self { req, context }
    }

    async fn run(self) {
        let ctx = self.context.clone();

        match self.req {
            FrontendMessage::Id(id, req) => {
                let resp = match req {
                    IdFrontendMessage::Cpu => {
                        let data = call_blocking_getter(ctx, getters::cpu).await;
                        IdBackendMessage::Cpu(data)
                    }
                    IdFrontendMessage::Temp => {
                        let data = call_blocking_getter(ctx, getters::temp).await;
                        IdBackendMessage::Temp(data)
                    }
                    IdFrontendMessage::Mem => {
                        let data = call_blocking_getter(ctx, getters::memory).await;
                        IdBackendMessage::Mem(data)
                    }
                    IdFrontendMessage::Disk => {
                        let data = call_blocking_getter(ctx, getters::disks).await;
                        IdBackendMessage::Disk(data)
                    }
                    IdFrontendMessage::NetIO => {
                        let data = call_blocking_getter(ctx, getters::network_io).await;
                        IdBackendMessage::NetIO(data)
                    }
                };

                let resp = BackendMessage::Id(id, resp);
                let _ = self.context.socket_tx.send(resp);
            }
            FrontendMessage::NoId(NoIdFrontendMessage::Terminal(msg)) => {
                let _ = self.context.term_tx.send(msg);
            }
        }
    }
}
