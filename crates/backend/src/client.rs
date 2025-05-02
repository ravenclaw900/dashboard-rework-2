use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use config::{PROTOCOL_VERSION, backend::BackendConfig};
use proto::{
    DashboardSocket,
    backend::{BackendMessage, Handshake, IdBackendMessage, NoIdBackendMessage},
    frontend::{FrontendMessage, IdFrontendMessage},
};
use sysinfo::{Components, Disks, Networks, System};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{SharedConfig, getters};

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
    config: SharedConfig,
    system: SharedSystem,
}

impl BackendContext {
    pub fn system(&mut self) -> impl std::ops::DerefMut<Target = SystemComponents> {
        self.system.lock().unwrap()
    }

    pub fn config(&self) -> &BackendConfig {
        &self.config
    }
}

pub struct BackendClient {
    socket: DashboardSocket,
    context: BackendContext,
}

impl BackendClient {
    pub async fn new(config: SharedConfig, system: SharedSystem) -> Result<Self> {
        let stream = TcpStream::connect(config.frontend_addr)
            .await
            .context("failed to connect to frontend")?;

        Ok(Self {
            socket: DashboardSocket::new(stream),
            context: BackendContext { config, system },
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        self.send_handshake().await?;

        loop {
            tokio::select! {
                frame_result = self.socket.read_frame() => {
                    let req: FrontendMessage = frame_result
                        .context("failed to read frame from frontend")?
                        .context("frontend unexpectedly disconnected")?;

                    let handler = RequestHandler::new(tx.clone(), req, self.context.clone());
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
    tx: mpsc::UnboundedSender<BackendMessage>,
    req: FrontendMessage,
    context: BackendContext,
}

impl RequestHandler {
    fn new(
        tx: mpsc::UnboundedSender<BackendMessage>,
        req: FrontendMessage,
        context: BackendContext,
    ) -> Self {
        Self { tx, req, context }
    }

    async fn run(self) {
        let ctx = self.context.clone();

        let resp = match self.req {
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

                BackendMessage::Id(id, resp)
            }
        };

        let _ = self.tx.send(resp);
    }
}
