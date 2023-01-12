use std::{fs::OpenOptions, path::PathBuf};

use anyhow::Result;
use dotenvy_macro::dotenv;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub online: bool,
    pub account: String,
    pub address: String,
    pub discord_channel: String,
    pub discord_token: String,
    pub discord_webhook: String,
    pub bed: Coordinates,
    pub pearl: Coordinates,
    pub bots: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            online: true,
            account: dotenv!("ACCOUNT").into(),
            address: dotenv!("ADDRESS").into(),
            discord_channel: dotenv!("DISCORD_CHANNEL").into(),
            discord_token: dotenv!("DISCORD_TOKEN").into(),
            discord_webhook: dotenv!("DISCORD_WEBHOOK").into(),
            bed: Default::default(),
            pearl: Default::default(),
            bots: Default::default(),
        }
    }
}

impl Config {
    pub fn get_path() -> Result<PathBuf> {
        let mut config_path = std::env::current_exe()?;

        config_path.set_file_name("config");
        config_path.set_extension("json");

        Ok(config_path)
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_path()?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_path)?;

        match serde_json::from_reader(&file) {
            Ok(config) => Ok(config),
            Err(_) => {
                let config = Config::default();
                serde_json::to_writer_pretty(&file, &config)?;

                Ok(config)
            }
        }
    }

    pub fn save(&mut self) -> Result<()> {
        let config_path = Self::get_path()?;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_path)?;

        serde_json::to_writer_pretty(&file, &self)?;

        Ok(())
    }
}
