use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use client::BackendClient;
use config::{VERSION, backend::get_config};
use log::{error, info};
use simple_logger::SimpleLogger;
use sysinfo::System;

mod client;
mod getters;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = get_config().context("failed to get config")?;
    let config = Box::leak(Box::new(config));

    SimpleLogger::new()
        .with_level(config.log_level)
        .init()
        .unwrap();

    info!("Starting DietPi-Dashboard backend v{VERSION}...");

    info!("Connecting to {}", config.frontend_addr);

    let system = Arc::new(Mutex::new(System::new()));

    loop {
        let client = BackendClient::new(config.frontend_addr, system.clone()).await?;

        if let Err(err) = client.run().await {
            error!("Connection error: {err:#}");
        }
    }
}
