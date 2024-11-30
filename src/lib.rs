#![feature(trivial_bounds)]

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_regex;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate str_macro;
#[macro_use]
extern crate strum;
#[macro_use]
extern crate tracing;

pub mod commands;
pub mod encryption;
pub mod plugins;
pub mod settings;
pub mod trapdoors;

use std::{collections::HashMap, io::ErrorKind, sync::Arc, time::Duration};

use anyhow::{bail, Result};
use azalea::{
    ecs::prelude::*,
    prelude::*,
    protocol::resolver,
    swarm::prelude::*,
    Account,
    JoinOpts,
};
use bevy_discord::bot::{DiscordBotConfig, DiscordBotPlugin};
use derive_config::{ConfigError, DeriveTomlConfig, DeriveYamlConfig};
use parking_lot::RwLock;
use semver::Version;
use serenity::prelude::*;
use smart_default::SmartDefault;
use terminal_link::Link;
use url::Url;
use uuid::Uuid;

pub use crate::{
    commands::handlers::prelude::*,
    plugins::{prelude::*, ShaysPluginGroup},
    settings::Settings,
    trapdoors::{Trapdoor, Trapdoors},
};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CARGO_PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
pub const CARGO_PKG_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// # Get the remote version using GitHub's latest release link redirect
///
/// # Errors
/// Will return `Err` if `ureq::get` fails.
pub fn get_remote_version() -> Result<Version> {
    let response = ureq::get(CARGO_PKG_HOMEPAGE).call()?;

    if let Ok(parsed_url) = Url::parse(response.get_url()) {
        if let Some(segments) = parsed_url.path_segments() {
            if let Some(remote_version) = segments.last() {
                return Ok(remote_version.parse()?);
            }
        }
    }

    bail!("Failed to get the remote version")
}

/// # Check for updates using GitHub's latest release link redirect
///
/// # Errors
/// Will return `Err` if `ureq::get` fails.
pub fn check_for_updates() -> Result<bool> {
    let local_version = CARGO_PKG_VERSION.parse()?;
    let remote_version = get_remote_version()?;

    Ok(remote_version > local_version)
}

/// # Load the Config or default if missing
///
/// # Errors
/// Will return `Err` if there's an error other than the file missing.
pub fn unwrap_or_else_default_if_not_found<C, D, L>(
    load_fn: L,
    default_fn: D,
) -> Result<C, ConfigError>
where
    D: FnOnce() -> C,
    L: FnOnce() -> Result<C, ConfigError>,
{
    match load_fn() {
        Ok(config) => Ok(config),
        Err(error) => match error {
            ConfigError::Io(error) if error.kind() == ErrorKind::NotFound => Ok(default_fn()),
            error => Err(error),
        },
    }
}

/// # Create and start the Minecraft bot client
///
/// # Errors
/// Will return `Err` if `ClientBuilder::start` fails.
#[allow(clippy::future_not_send)]
pub async fn start() -> Result<()> {
    let settings = unwrap_or_else_default_if_not_found(Settings::load, Settings::default)?;
    let trapdoors = unwrap_or_else_default_if_not_found(Trapdoors::load, Trapdoors::default)?;
    let mut client = SwarmBuilder::new().add_plugins((ShaysPluginGroup, MinecraftCommandsPlugin));
    settings.save()?; /* Save the settings before anything else */

    /* Check for updates after loading files to reduce web request spam */
    if check_for_updates()? {
        let version = get_remote_version()?;
        let text = format!("An update is available: {CARGO_PKG_REPOSITORY}/releases/tag/{version}");
        let link = Link::new(&text, CARGO_PKG_HOMEPAGE);
        info!("{link}");
    }

    /* Enable the Discord plugin if a token was provided */
    if !settings.discord_token.is_empty() {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let config = DiscordBotConfig::default()
            .gateway_intents(intents)
            .token(settings.discord_token.clone());

        client = client.add_plugins((
            DiscordBotPlugin::new(config),
            DiscordCommandsPlugin,
            DiscordEventLoggerPlugin,
        ));
    }

    /* Add each account, possibly with its own proxy server address */
    for bot_settings in settings.locations.values() {
        let account = if bot_settings.online_mode {
            Account::microsoft(&bot_settings.account_username).await?
        } else {
            Account::offline(&bot_settings.account_username)
        };

        client = if let Some(server_address) = bot_settings.server_address.clone() {
            let Ok(resolved_address) = resolver::resolve_address(&server_address).await else {
                bail!("Failed to resolve server address")
            };

            let opts = JoinOpts::new()
                .custom_address(server_address)
                .custom_resolved_address(resolved_address);

            client.add_account_with_opts(account, opts)
        } else {
            client.add_account(account) /* Use the default server address */
        };
    }

    /* Clone the address before giving ownership of the settings */
    let address = settings.server_address.clone();

    client
        .add_plugins((settings, trapdoors))
        .join_delay(Duration::from_secs(5))
        .set_swarm_handler(swarm_handler)
        .start(address)
        .await?
}

#[derive(Clone, Component, Resource, SmartDefault)]
pub struct SwarmState {
    auto_reconnect: Arc<RwLock<HashMap<Uuid, bool>>>,
}

/// # Errors
/// Will return `Err` if `Swarm::add_with_opts` fails.
pub async fn swarm_handler(mut swarm: Swarm, event: SwarmEvent, state: SwarmState) -> Result<()> {
    match event {
        SwarmEvent::Login => {}
        SwarmEvent::Init => swarm.ecs_lock.lock().insert_resource(state),
        SwarmEvent::Chat(chat_packet) => info!("{}", chat_packet.message().to_ansi()),
        SwarmEvent::Disconnect(ref account, ref join_opts) => loop {
            let uuid = account.uuid_or_offline();
            if !state.auto_reconnect.read().get(&uuid).unwrap_or(&true) {
                continue;
            }

            info!("[AutoReconnect] Reconnecting in 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
            match swarm.add_with_opts(account, state.clone(), join_opts).await {
                Err(error) => error!("Error: {error}"),
                Ok(_) => break,
            }
        },
    }

    Ok(())
}
