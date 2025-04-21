use std::{fs, io};

use anyhow::{Context, Result};
use toml_migrate::{ConfigMigrator, Migrate};

#[cfg(feature = "backend")]
pub mod backend;
#[cfg(feature = "frontend")]
pub mod frontend;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROTOCOL_VERSION: u32 = 1;

macro_rules! generate_config_file {
    ($template:literal, $($key:ident = $val:expr),*) => {{
        use serde::Serialize;
        use toml_edit::ser::ValueSerializer;

        $( let $key = Serialize::serialize(&($val), ValueSerializer::new()).unwrap(); )*

        format!(include_str!(concat!("../templates/", $template)), $($key = $key),*)
    }};
}

pub(crate) use generate_config_file;

fn read_config<T: Migrate + Default>(
    config_name: &str,
    config_file_generator: fn(&T) -> String,
) -> Result<T> {
    let mut cfgpath = std::env::current_exe().context("couldn't get path to executable")?;
    cfgpath.set_file_name(config_name);

    let config_str = match std::fs::read_to_string(&cfgpath) {
        Ok(config_str) => config_str,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // If config file doesn't exist, create a new default configuration
            let config = T::default();
            let config_file = config_file_generator(&config);
            fs::write(cfgpath, config_file).context("failed to create new config file")?;
            return Ok(config);
        }
        Err(e) => return Err(e).context("failed to read config file"),
    };

    let migrator = ConfigMigrator::new("CONFIG_VERSION_DO_NOT_CHANGE").with_default_version(0);
    let (config, migration_occurred) = migrator
        .migrate_config(&config_str)
        .context("failed to migrate config file")?;

    if migration_occurred {
        let config_file = config_file_generator(&config);
        fs::write(cfgpath, config_file).context("failed to write updated config file")?;
    }

    Ok(config)
}
