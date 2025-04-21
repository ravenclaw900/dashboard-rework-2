use std::{
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
};

use anyhow::{Context, Result};
use flexible_hyper_server_tls::{HttpOrHttpsAcceptor, rustls_helpers};
use hyper::service::service_fn;
use log::{error, info};
use request::ServerRequest;
use router::router;
use tokio::net::TcpListener;

use crate::backend::SharedBackendRegistry;

pub mod request;
pub mod response;
mod router;
mod statics;

pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

pub struct HttpServer {
    acceptor: HttpOrHttpsAcceptor,
    backends: SharedBackendRegistry,
}

impl HttpServer {
    pub async fn new(
        port: u16,
        tls_config: Option<TlsConfig>,
        backends: SharedBackendRegistry,
    ) -> Result<Self> {
        info!("Starting web server on port {port}");

        let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));
        let listener = TcpListener::bind(addr)
            .await
            .context("failed to bind http server")?;

        let mut acceptor = HttpOrHttpsAcceptor::new(listener);

        if let Some(tls_config) = tls_config {
            let tls = rustls_helpers::get_tlsacceptor_from_files(
                tls_config.cert_path,
                tls_config.key_path,
            )
            .await
            .context("failed to build TlsAcceptor")?;

            acceptor = acceptor.with_tls(tls)
        }

        Ok(Self { acceptor, backends })
    }

    pub async fn run(self) {
        loop {
            let backends = self.backends.clone();

            let service = service_fn(move |req| {
                let req = ServerRequest::new(req, backends.clone());
                async move { router(req).await }
            });

            if let Ok((_, conn_fut)) = self.acceptor.accept(service).await {
                tokio::task::spawn_local(async move {
                    if let Err(err) = conn_fut.await {
                        error!("Error serving HTTP connection: {err}");
                    }
                });
            }
        }
    }
}
