use std::{cell::RefCell, rc::Rc};

use anyhow::{Context, Result};
use backend::{BackendRegistry, BackendServer};
use config::{VERSION, frontend::get_config};
use http::{HttpServer, TlsConfig};
use log::info;
use simple_logger::SimpleLogger;
use tokio::task::LocalSet;

mod backend;
mod http;
mod pages;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Using a LocalSet until LocalRuntime stabilizes
    let local_set = LocalSet::new();

    let config = get_config().context("failed to get config")?;

    SimpleLogger::new()
        .with_level(config.log_level)
        .init()
        .unwrap();

    info!("Starting DietPi-Dashboard frontend v{VERSION}...");

    let backends = Rc::new(RefCell::new(BackendRegistry::new()));

    let backend_server = BackendServer::new(config.backend_port, backends.clone()).await?;
    local_set.spawn_local(backend_server.run());

    let tls_config = config.enable_tls.then_some(TlsConfig {
        cert_path: config.cert_path,
        key_path: config.key_path,
    });

    let http_server = HttpServer::new(config.http_port, tls_config, backends.clone()).await?;
    local_set.spawn_local(http_server.run());

    local_set.await;

    Ok(())
}
