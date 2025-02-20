use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use log::{error, info, warn};
use proto::{
    types::{Handshake, Request},
    DashboardSocket, Frame,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

#[derive(Debug)]
pub struct BackendInfo {
    pub nickname: String,
    pub handle: BackendHandle,
}

pub type BackendRegistry = HashMap<IpAddr, BackendInfo>;
pub type SharedBackendRegistry = Arc<Mutex<BackendRegistry>>;

pub struct BackendServer {
    listener: TcpListener,
    registry: SharedBackendRegistry,
}

impl BackendServer {
    pub async fn new(port: u16, registry: SharedBackendRegistry) -> Result<Self> {
        info!("Starting backend server on port {port}");

        let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));
        let listener = TcpListener::bind(addr)
            .await
            .context("failed to bind backend tcp server")?;

        Ok(Self { listener, registry })
    }

    pub async fn run(self) {
        loop {
            let (stream, peer_ip) = match self.listener.accept().await {
                Ok((stream, peer_addr)) => (stream, peer_addr.ip().to_canonical()),
                Err(err) => {
                    error!("Failed to accept backend connection: {err:#}");
                    continue;
                }
            };

            info!("New backend connection from {}", peer_ip);

            let conn = BackendConnection::new(stream, self.registry.clone(), peer_ip);

            tokio::spawn(conn.handle_connection());
        }
    }
}

struct BackendRequest {
    req: Request,
    resp_tx: oneshot::Sender<Vec<u8>>,
}

struct BackendConnection {
    socket: DashboardSocket,
    registry: SharedBackendRegistry,
    addr: IpAddr,
}

impl BackendConnection {
    fn new(stream: TcpStream, registry: SharedBackendRegistry, addr: IpAddr) -> Self {
        Self {
            socket: DashboardSocket::new(stream),
            registry,
            addr,
        }
    }

    async fn handle_connection(mut self) {
        let (tx, rx) = mpsc::unbounded_channel();

        let handshake = match self.read_handshake().await {
            Ok(handshake) => handshake,
            Err(err) => {
                error!("Handshake with backend {} failed: {err:#}", self.addr);
                return;
            }
        };

        let nickname = self.determine_nickname(handshake);

        let conn_info = BackendInfo {
            nickname,
            handle: BackendHandle::new(tx),
        };

        self.registry.lock().unwrap().insert(self.addr, conn_info);

        if let Err(err) = self.handle_requests(rx).await {
            error!("Error handling requests for backend {}: {err:#}", self.addr)
        }

        self.registry.lock().unwrap().remove(&self.addr);
    }

    async fn read_frame(&mut self) -> Result<Option<Frame>> {
        self.socket
            .read_frame()
            .await
            .context("failed to read frame from socket")
    }

    async fn read_handshake(&mut self) -> Result<Handshake> {
        let (_, handshake) = self
            .read_frame()
            .await
            .and_then(|opt| opt.context("peer disconnected before sending handshake"))
            .and_then(|frame: Frame| {
                frame
                    .into_decode::<Handshake>()
                    .context("failed to decode handshake")
            })?;

        Ok(handshake)
    }

    fn determine_nickname(&self, handshake: Handshake) -> String {
        if !handshake.nickname.is_empty() {
            handshake.nickname
        } else {
            self.addr.to_string()
        }
    }

    async fn handle_requests(
        &mut self,
        mut rx: mpsc::UnboundedReceiver<BackendRequest>,
    ) -> Result<()> {
        let mut next_id = 0;
        let mut in_progress: HashMap<u16, oneshot::Sender<Vec<u8>>> = HashMap::new();

        loop {
            tokio::select! {
                chan_result = rx.recv() => {
                    let Some(conn_req) = chan_result else {
                        break;
                    };

                    let frame = Frame::from_encode(next_id, &conn_req.req);

                    self.socket
                        .write_frame(frame)
                        .await
                        .context("failed to write request frame")?;

                    in_progress.insert(next_id, conn_req.resp_tx);

                    next_id += 1;
                }
                frame_result = self.read_frame() => {
                    let Some(frame) = frame_result? else {
                        info!("Backend {} disconnected", self.addr);
                        break;
                    };

                    let (id, data) = frame.into_data();

                    let Some(resp_tx) = in_progress.remove(&id) else {
                        warn!("Received frame with unknown id {id} from {}", self.addr);
                        continue;
                    };

                    let _ = resp_tx.send(data);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BackendHandle {
    tx: mpsc::UnboundedSender<BackendRequest>,
}

impl BackendHandle {
    fn new(tx: mpsc::UnboundedSender<BackendRequest>) -> Self {
        Self { tx }
    }

    pub async fn send_req<Resp: bitcode::DecodeOwned>(&self, req: Request) -> Result<Resp> {
        let (resp_tx, resp_rx) = oneshot::channel();

        self.tx
            .send(BackendRequest { req, resp_tx })
            .context("failed to send request, connection likely closed")?;

        let resp_data = resp_rx
            .await
            .context("failed to recv response, connection likely closed")?;

        let resp = bitcode::decode(&resp_data).context("failed to decode response")?;

        Ok(resp)
    }
}
