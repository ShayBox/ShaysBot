#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
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

pub mod prelude;

pub mod commands;
pub mod modules;
pub mod parsers;
pub mod settings;
pub mod trackers;

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{bail, Result};
use azalea::{ecs::prelude::*, prelude::*, swarm::prelude::*};
use azalea_viaversion::ViaVersionPlugin;
#[cfg(feature = "bot")]
use bevy_discord::config::DiscordBotConfig;
use bevy_discord::DiscordPluginGroup;
use parking_lot::RwLock;
use semver::Version;
#[cfg(feature = "bot")]
use serenity::prelude::*;
use smart_default::SmartDefault;
use terminal_link::Link;
use ureq::ResponseExt;

use crate::prelude::*;

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CARGO_PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
pub const CARGO_PKG_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// # Get the remote version using GitHub's latest release link redirect
///
/// # Errors
/// Will return `Err` if `ureq::get` fails.
pub fn get_remote_version() -> Result<Version> {
    let response = ureq::get(CARGO_PKG_HOMEPAGE).call()?;

    let url = response.get_uri().to_string();
    if let Some(remote_version) = url.split('/').next_back() {
        return Ok(remote_version.parse()?);
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

/// # Create and start the Minecraft bot client
///
/// # Errors
/// Will return `Err` if `ClientBuilder::start` fails.
#[allow(clippy::future_not_send)]
pub async fn start() -> Result<()> {
    let global_settings = GlobalSettings::load()?.save()?;
    let mut client = SwarmBuilder::new()
        .set_swarm_handler(swarm_handler)
        .add_plugins((
            CommandsPluginGroup,
            MinecraftParserPlugin,
            ModulesPluginGroup,
            SettingsPluginGroup,
            TrackersPluginGroup,
        ));

    /* Check for updates after loading files to reduce web request spam */
    if check_for_updates()? {
        let version = get_remote_version()?;
        let text = format!("An update is available: {CARGO_PKG_REPOSITORY}/releases/tag/{version}");
        let link = Link::new(&text, CARGO_PKG_HOMEPAGE);
        info!("{link}");
    }

    #[cfg(feature = "api")]
    if global_settings.http_api.enabled {
        client = client.add_plugins(HttpApiParserPlugin);
    }

    #[cfg(feature = "bot")]
    if !global_settings.discord_token.is_empty() {
        let gateway_intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let bot_config = DiscordBotConfig::default()
            .gateway_intents(gateway_intents)
            .token(global_settings.discord_token.clone());

        client = client.add_plugins((
            DiscordPluginGroup { bot_config },
            DiscordParserPlugin,
            DiscordLoggerPlugin,
        ));
    }

    /* ViaProxy for multi-version compatibility */
    if !global_settings.server_version.is_empty() {
        client = client.add_plugins(ViaVersionPlugin::start(&global_settings.server_version).await);
    }

    client.start(global_settings.server_address).await?
}

#[derive(Clone, Component, Resource, SmartDefault)]
pub struct SwarmState {
    auto_reconnect: Arc<RwLock<HashMap<String, (bool, u64)>>>,
}

/// # Errors
/// Will return `Err` if `Swarm::add_with_opts` fails.
pub async fn swarm_handler(swarm: Swarm, event: SwarmEvent, state: SwarmState) -> Result<()> {
    match event {
        SwarmEvent::Login => {}
        SwarmEvent::Init => swarm.ecs_lock.lock().insert_resource(state),
        SwarmEvent::Chat(chat_packet) => {
            let message = chat_packet.message();
            if message.to_string().contains("Position in queue: ") {
                return Ok(()); /* 2B2T Queue */
            }

            println!("{}", message.to_ansi());
        }
        SwarmEvent::Disconnect(ref account, ref join_opts) => loop {
            let bot_name = account.username.to_lowercase();
            let Some((rejoin, secs)) = state.auto_reconnect.read().get(&bot_name).copied() else {
                state
                    .auto_reconnect
                    .write()
                    .insert(bot_name.to_lowercase(), (false, 5));

                continue; /* AutoReconnect: Missing */
            };

            tokio::time::sleep(Duration::from_secs(secs)).await;

            if !rejoin {
                continue; /* AutoReconnect: Disabled */
            }

            info!("AutoReconnecting on {}", account.username);
            if let Err(reason) = swarm.add_with_opts(account, state.clone(), join_opts).await {
                warn!("[{bot_name}] Failed to AutoReconnect: {reason}");
                info!("[{bot_name}] AutoReconnecting in 30s...");

                state.auto_reconnect.write().entry(bot_name).or_default().1 = 30;
                continue;
            }

            break;
        },
    }

    Ok(())
}
