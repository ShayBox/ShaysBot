use std::{
    collections::HashMap,
    fs::File,
    io::{ErrorKind, Read, Seek, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use azalea::{
    app::{App, Plugin},
    prelude::*,
    protocol::address::ServerAddr,
};
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple as DeserializeTuple, Serialize_tuple as SerializeTuple};
use serde_with::DurationSeconds;
use smart_default::SmartDefault;
use uuid::Uuid;

/// Global Swarm Settings that apply to every account
pub struct GlobalSettingsPlugin;

impl Plugin for GlobalSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalSettings::load().expect("Failed to load global settings"));
    }
}

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

    /// Discord client token for commands and responses. (Optional)
    pub discord_token: String,

    /// Logger configuration for sending game events to Discord via webhooks.
    #[serde(default)]
    pub logger: LoggerConfig,

    /// Minecraft server ender pearl view distance in blocks.
    /// Better to under-estimate than to over-estimate.
    #[default(60)] /* Vanilla/Spigot/Paper/Folia Default */
    pub pearl_view_distance: i32,

    /// Minecraft server address.
    #[default(ServerAddr{
        host: str!("play.vengeancecraft.net"),
        port: 25565
    })]
    pub server_address: ServerAddr,

    /// `ViaProxy` server version. (Optional)
    pub server_version: String,

    /// Automatically whitelist players that enter visual range.
    #[default(false)]
    pub whitelist_in_range: bool,

    /// Disable commands for non-whitelisted players.
    #[default(false)]
    pub whitelist_only: bool,

    /// API Server for local integrations.
    #[cfg(feature = "api")]
    #[serde(rename = "api_server")]
    pub http_api: ApiServer,

    /// Chat encryption using the NCR (No Chat Reports) mod.
    #[serde(rename = "chat_encryption")]
    pub chat: ChatEncryption,

    /// Minecraft accounts with their linked Discord ID and API Password.
    pub users: HashMap<Uuid, User>,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct ApiServer {
    #[default(false)]
    pub enabled: bool,

    /// API Server bind address. (default local only & random port)
    #[default("127.0.0.1:0")]
    pub bind_addr: String,
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

#[serde_as]
#[derive(Clone, Default, Eq, PartialEq, DeserializeTuple, SerializeTuple)]
pub struct User {
    pub discord_id:   String,
    pub api_password: String,
}

/// Logger configuration for sending game events to Discord via webhooks.
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct LoggerConfig {
    /// Global list of webhook URLs used as the default for all event types.
    pub webhooks: Vec<String>,

    /// Per-event-type configuration.
    #[serde(default)]
    pub event: EventTypes,
}

/// Container for all per-event webhook configurations.
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct EventTypes {
    /// Player join events (when a bot joins the game).
    pub player_join: WebhookEventConfig,

    /// Player leave events (when a bot leaves the game).
    pub player_leave: WebhookEventConfig,

    /// Player visual range enter events.
    pub player_enter: WebhookEventConfig,

    /// Player visual range exit events.
    pub player_exit: WebhookEventConfig,

    /// Command execution events by players.
    pub player_command: WebhookEventConfig,

    /// Ender pearl pull events.
    pub player_pearl: WebhookEventConfig,

    /// Block break events in visual range.
    pub player_break: WebhookBlockEventConfig,

    /// Block place events in visual range.
    pub player_place: WebhookBlockEventConfig,

    /// Ender pearl inventory depleted at stasis chamber.
    pub pearl_missing: WebhookEventConfig,

    /// Pearl goto pathfinding failed (pathfinder busy).
    pub pearl_path_failed: WebhookEventConfig,

    /// Pearl return to idle goal after pulling.
    pub pearl_return: WebhookEventConfig,

    /// Auto-whitelist adds a player to the whitelist.
    pub auto_whitelist_add: WebhookEventConfig,

    /// Chat messages received from other players.
    pub player_chat: WebhookEventConfig,

    /// Server-side disconnect (reason from server).
    pub server_disconnect: WebhookEventConfig,

    /// Successful reconnection after disconnect.
    pub server_reconnect: WebhookEventConfig,

    /// Connection errors (timeout, auth failure, etc.).
    pub server_error: WebhookEventConfig,
}

/// Configuration for a single event type's webhook logging.
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct WebhookEventConfig {
    /// Whether to log this event type.
    #[default(true)]
    pub enabled: bool,

    /// Optional override list of webhook URLs for this specific event type.
    /// If not specified, falls back to the global `webhooks` list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Vec<String>>,
}

/// Configuration for block-related events with a configurable block filter.
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct WebhookBlockEventConfig {
    /// Whether to log this event type.
    #[default(true)]
    pub enabled: bool,

    /// Optional override list of webhook URLs for this specific event type.
    /// If not specified, falls back to the global `webhooks` list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Vec<String>>,

    /// Block IDs or names to log (e.g., `"minecraft:shulker_box"`, `"netherite_block"`).
    /// If empty, uses the default high-value block list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<String>>,
}

/// Default list of high-value blocks to log when no custom list is provided.
pub const DEFAULT_BLOCK_FILTER: &[&str] = &[
    "shulker_box",
    "black_shulker_box",
    "blue_shulker_box",
    "brown_shulker_box",
    "cyan_shulker_box",
    "gray_shulker_box",
    "green_shulker_box",
    "light_blue_shulker_box",
    "light_gray_shulker_box",
    "lime_shulker_box",
    "magenta_shulker_box",
    "orange_shulker_box",
    "pink_shulker_box",
    "purple_shulker_box",
    "red_shulker_box",
    "white_shulker_box",
    "yellow_shulker_box",
    "netherite_block",
    "gold_block",
    "diamond_block",
    "emerald_block",
    "lapis_block",
    "redstone_block",
];

/// Check if a block ID matches the configured filter.
pub fn is_logged_block(block_id: &str, custom_blocks: Option<&[String]>) -> bool {
    match custom_blocks {
        Some(list) => list.iter().any(|filter| block_id.contains(filter.as_str())),
        None => DEFAULT_BLOCK_FILTER.iter().any(|filter| block_id.contains(*filter)),
    }
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
    /// Will return `Err` if `std::env::current_exe` or `std::env::current_dir` fails.
    pub fn path() -> Result<PathBuf> {
        let path = if cfg!(debug_assertions) {
            let path = std::env::current_exe()?;
            path.parent().context("None")?.to_path_buf()
        } else {
            std::env::current_dir()?
        };

        Ok(path.join("global-settings.toml"))
    }

    /// # Errors
    /// Will return `Err` if `File::open`, `toml::to_string_pretty`, or `File::write_all` fails.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        match File::open(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
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
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let text = toml::to_string(&self)?;
        let buf = text.as_bytes();
        file.write_all(buf)?;

        Ok(())
    }
}
