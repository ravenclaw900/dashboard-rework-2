use std::{collections::HashMap, net::IpAddr};

use anyhow::{Context, Result, anyhow};
use config::PROTOCOL_VERSION;
use log::{error, info, warn};
use proto::{
    DashboardSocket, // types::{BackendMessageType, FrontendMessage},
    backend::{BackendMessage, Handshake, IdBackendMessage, NoIdBackendMessage},
    frontend::{FrontendMessage, IdFrontendMessage},
};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use super::{SharedBackendRegistry, cache::BackendCache};

#[derive(Debug)]
pub struct BackendInfo {
    pub nickname: String,
    pub handle: BackendHandle,
}

enum BackendRequest {
    WithResp {
        req: IdFrontendMessage,
        resp_tx: oneshot::Sender<IdBackendMessage>,
    },
}

pub struct BackendConnection {
    socket: DashboardSocket,
    registry: SharedBackendRegistry,
    addr: IpAddr,
}

impl BackendConnection {
    pub fn new(stream: TcpStream, registry: SharedBackendRegistry, addr: IpAddr) -> Self {
        Self {
            socket: DashboardSocket::new(stream),
            registry,
            addr,
        }
    }

    pub async fn handle_connection(mut self) {
        let (tx, rx) = mpsc::unbounded_channel();

        let handshake = match self.read_handshake().await {
            Ok(handshake) => handshake,
            Err(err) => {
                error!("Handshake with backend {} failed: {err:#}", self.addr);
                return;
            }
        };

        if handshake.version != PROTOCOL_VERSION {
            warn!("Backend with incompatable version connected");
            return;
        }

        let nickname = if !handshake.nickname.is_empty() {
            handshake.nickname
        } else {
            self.addr.to_string()
        };

        let conn_info = BackendInfo {
            nickname,
            handle: BackendHandle::new(tx),
        };

        self.registry.borrow_mut().insert(self.addr, conn_info);

        if let Err(err) = self.handle_requests(rx).await {
            error!("Error handling requests for backend {}: {err:#}", self.addr)
        }

        self.registry.borrow_mut().remove(&self.addr);
    }

    async fn read_frame(&mut self) -> Result<Option<BackendMessage>> {
        self.socket
            .read_frame()
            .await
            .context("failed to read frame from socket")
    }

    async fn read_handshake(&mut self) -> Result<Handshake> {
        let message = self
            .read_frame()
            .await
            .and_then(|opt| opt.context("peer disconnected before sending handshake"))?;
        let BackendMessage::NoId(NoIdBackendMessage::Handshake(handshake)) = message else {
            return Err(anyhow!("peer sent invalid message, expected handshake"));
        };

        Ok(handshake)
    }

    async fn handle_requests(
        &mut self,
        mut rx: mpsc::UnboundedReceiver<BackendRequest>,
    ) -> Result<()> {
        let mut next_id = 0;
        let mut in_progress: HashMap<u16, oneshot::Sender<IdBackendMessage>> = HashMap::new();
        let mut cache = BackendCache::new();

        loop {
            tokio::select! {
                chan_result = rx.recv() => {
                    let Some(conn_req) = chan_result else {
                        break;
                    };

                    match conn_req {
                        BackendRequest::WithResp {req, resp_tx} => {
                            if let Some(data) = cache.get(&req) {
                                let _ = resp_tx.send(data);
                                continue;
                            }

                            let msg = FrontendMessage::Id(next_id, req);

                            self.socket
                                .write_frame(msg)
                                .await
                                .context("failed to write request frame")?;

                            // Save response channel so we can send to it when we receive a response
                            in_progress.insert(next_id, resp_tx);

                            next_id += 1;
                        }
                    }
                }
                resp_result = self.read_frame() => {
                    let Some(resp) = resp_result? else {
                        info!("Backend {} disconnected", self.addr);
                        break;
                    };

                    match resp {
                        BackendMessage::Id(id, data) => {
                            let Some(resp_tx) = in_progress.remove(&id) else {
                                warn!("Received frame with unknown id {} from {}", id, self.addr);
                                continue;
                            };

                            cache.insert(data.clone());

                            let _ = resp_tx.send(data);
                        },
                        BackendMessage::NoId(_) => {
                            warn!("Received unexpected message from backend {} (possibly extraneous handshake)", self.addr);
                            continue;
                        }
                    }
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

    pub async fn send_req_with_resp(&self, req: IdFrontendMessage) -> Result<IdBackendMessage> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let req = BackendRequest::WithResp { req, resp_tx };

        self.tx
            .send(req)
            .context("failed to send request, connection likely closed")?;

        let resp = resp_rx
            .await
            .context("failed to recv response, connection likely closed")?;

        Ok(resp)
    }
}
