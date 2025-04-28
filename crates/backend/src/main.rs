use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use client::{BackendClient, SystemComponents};
use config::{
    APP_VERSION,
    backend::{BackendConfig, get_config},
};
use log::{error, info};
use simple_logger::SimpleLogger;

mod client;
mod getters;

pub type SharedConfig = Arc<BackendConfig>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Arc::new(get_config().context("failed to get config")?);

    SimpleLogger::new()
        .with_level(config.log_level)
        .init()
        .unwrap();

    info!("Starting DietPi-Dashboard backend v{APP_VERSION}...");

    info!("Connecting to {}", config.frontend_addr);

    let system = Arc::new(Mutex::new(SystemComponents::new()));

    let client = BackendClient::new(config.clone(), system.clone()).await?;

    if let Err(err) = client.run().await {
        error!("Connection error: {err:#}");
    }

    Ok(())
}
