use std::{
    collections::HashMap,
    fs::File,
    io::{ErrorKind, Read, Seek, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{bail, Result};
use azalea::{
    app::{App, Plugin},
    prelude::*,
    protocol::ServerAddress,
};
use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, NoneAsEmptyString};
use smart_default::SmartDefault;
use uuid::Uuid;

#[serde_as]
#[derive(Clone, Deserialize, Serialize, SmartDefault, Resource)]
#[serde(default)]
pub struct GlobalSettings {
    /// Chat command prefix.
    #[default("!")]
    pub command_prefix: String,

    /// Command cooldown in seconds.
    #[default(Duration::from_secs(10))]
    #[serde_as(as = "DurationSeconds")]
    pub command_cooldown: Duration,

    /// Minecraft server ender pearl view distance in blocks.
    /// It is better to under-estimate than to over-estimate.
    #[default(64)]
    pub pearl_view_distance: i32,

    /// Minecraft server address.
    #[default(ServerAddress{
        host: str!("play.vengeancecraft.net"),
        port: 25565
        })]
    pub server_address: ServerAddress,

    /// ViaProxy server version. (Optional)
    #[default("")]
    pub server_version: String,

    /// Disable commands for non-whitelisted players.
    #[default(false)]
    pub whitelist: bool,

    /// API Server for local integrations.
    #[cfg(feature = "api")]
    #[serde(rename = "api_server")]
    pub api: ApiServer,

    /// Discord bot for commands and events.
    #[cfg(feature = "discord")]
    #[serde(rename = "discord_bot")]
    pub discord: DiscordBot,

    /// Chat encryption using the NCR (No Chat Reports) mod.
    #[serde(rename = "chat_encryption")]
    pub encryption: ChatEncryption,

    /// Whitelisted Minecraft accounts and their linked Discord accounts.
    #[serde_as(as = "HashMap<_, NoneAsEmptyString>")]
    pub whitelisted: HashMap<Uuid, Option<String>>,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct ApiServer {
    #[default(false)]
    pub enabled: bool,

    /// API Server bind address. (default localhost only, random port)
    #[default("127.0.0.1:0")]
    pub bind_addr: String,

    /// Authentication username
    #[default("username")]
    pub username: String,

    /// Authentication password
    #[default("password")]
    pub password: String,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct ChatEncryption {
    /// Encryption key (default is public)
    #[default("blfrngArk3chG6wzncOZ5A==")]
    pub key: String,

    /// Encryption response mode. (`OnDemand`, `Always`, or `Never`)
    #[default(EncryptionMode::OnDemand)]
    pub mode: EncryptionMode,
}

#[cfg(feature = "discord")]
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct DiscordBot {
    #[default(false)]
    pub enabled: bool,

    /// Discord client token.
    pub token: String,
}

#[derive(Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EncryptionMode {
    #[default]
    OnDemand,
    Always,
    Never,
}

impl GlobalSettings {
    /// # Errors
    /// Will return `Err` if `std::env::current_exe` fails.
    pub fn path() -> Result<PathBuf> {
        let mut path = if cfg!(debug_assertions) {
            std::env::current_exe()?
        } else {
            std::env::current_dir()?
        };

        path.set_file_name("global-settings");
        path.set_extension("toml");

        Ok(path)
    }

    /// # Errors
    /// Will return `Err` if `File::open`, `toml::to_string_pretty`, or `File::write_all` fails.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        match File::open(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Self::default().save(),
            Err(error) => bail!(error),
            Ok(mut file) => {
                let mut text = String::new();
                file.read_to_string(&mut text)?;
                file.rewind()?;

                Ok(toml::from_str(&text)?)
            }
        }
    }

    /// # Errors
    /// Will return `Err` if `File::open`, `File::read_to_string`, `File::rewind`, or `toml::from_str` fails.
    pub fn save(self) -> Result<Self> {
        let path = Self::path()?;
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let text = toml::to_string_pretty(&self)?;
        let buf = text.as_bytes();
        file.write_all(buf)?;

        Ok(self)
    }
}

/// Handle global swarm settings.
pub struct GlobalSettingsPlugin;

impl Plugin for GlobalSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalSettings::load().expect("Failed to load global settings"));
    }
}
