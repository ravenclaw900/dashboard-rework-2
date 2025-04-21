use std::{cell::RefCell, rc::Rc};

use anyhow::{Context, Result};
use backend::{BackendRegistry, BackendServer};
use config::{
    APP_VERSION,
    frontend::{FrontendConfig, get_config},
};
use http::HttpServer;
use log::info;
use simple_logger::SimpleLogger;
use tokio::task::LocalSet;

mod backend;
mod http;
mod pages;

pub type SharedConfig = Rc<FrontendConfig>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Using a LocalSet until LocalRuntime stabilizes
    let local_set = LocalSet::new();

    let config = Rc::new(get_config().context("failed to get config")?);

    SimpleLogger::new()
        .with_level(config.log_level)
        .init()
        .unwrap();

    info!("Starting DietPi-Dashboard frontend v{APP_VERSION}...");

    let backends = Rc::new(RefCell::new(BackendRegistry::new()));

    let backend_server = BackendServer::new(config.backend_port, backends.clone()).await?;
    local_set.spawn_local(backend_server.run());

    let http_server = HttpServer::new(config, backends.clone()).await?;
    local_set.spawn_local(http_server.run());

    local_set.await;

    Ok(())
}
