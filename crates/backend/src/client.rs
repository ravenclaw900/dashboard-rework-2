use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use config::{PROTOCOL_VERSION, backend::BackendConfig};
use proto::{
    DashboardSocket,
    types::{BackendMessage, BackendMessageType, FrontendMessage, FrontendMessageType, Handshake},
};
use sysinfo::{Components, Disks, Networks, System};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{SharedConfig, getters};

macro_rules! call_getter {
    (
        blocking,
        getter = $getter:path as $typ:path,
        ctx = $ctx:expr,
        id = $id:expr
    ) => {{
        let data = tokio::task::spawn_blocking(move || $getter($ctx))
            .await
            .unwrap();
        let data = $typ(data);
        BackendMessage { id: $id, data }
    }};
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

        let message = BackendMessage {
            id: None,
            data: BackendMessageType::Handshake(handshake),
        };

        self.socket
            .write_frame(message)
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
        let resp = match self.req.data {
            FrontendMessageType::Cpu => call_getter!(
                blocking,
                getter = getters::cpu as BackendMessageType::Cpu,
                ctx = self.context,
                id = self.req.id
            ),
            FrontendMessageType::Temp => call_getter!(
                blocking,
                getter = getters::temp as BackendMessageType::Temp,
                ctx = self.context,
                id = self.req.id
            ),
            FrontendMessageType::Mem => call_getter!(
                blocking,
                getter = getters::memory as BackendMessageType::Mem,
                ctx = self.context,
                id = self.req.id
            ),
            FrontendMessageType::Disk => {
                call_getter!(
                    blocking,
                    getter = getters::disks as BackendMessageType::Disk,
                    ctx = self.context,
                    id = self.req.id
                )
            }
            FrontendMessageType::NetIO => {
                call_getter!(
                    blocking,
                    getter = getters::network_io as BackendMessageType::NetIO,
                    ctx = self.context,
                    id = self.req.id
                )
            }
        };

        let _ = self.tx.send(resp);
    }
}
