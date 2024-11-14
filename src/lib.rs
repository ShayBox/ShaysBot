#![feature(trivial_bounds)]

#[macro_use]
extern crate lazy_regex;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate structstruck;
#[macro_use]
extern crate strum;
#[macro_use]
extern crate tracing;

pub mod commands;
pub mod encryption;
pub mod plugins;
pub mod settings;
pub mod trapdoors;

use std::ops::{AddAssign, RemAssign};

use anyhow::{bail, Result};
use azalea::{
    ecs::prelude::*,
    prelude::*,
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
    Account,
};
use bevy_discord::bot::{DiscordBotConfig, DiscordBotPlugin};
use derive_config::{DeriveTomlConfig, DeriveYamlConfig};
use num_traits::{Bounded, One};
use semver::Version;
use serenity::{all::ChannelId, prelude::*};
use url::Url;

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

#[derive(Clone, Component, Default, Resource)]
pub struct SwarmState;

/// # Create and start the Minecraft bot client
///
/// # Errors
/// Will return `Err` if `ClientBuilder::start` fails.
#[allow(clippy::future_not_send)]
pub async fn start() -> Result<()> {
    let settings = Settings::load().unwrap_or_else(|error| {
        error!("Error loading settings: {error}");
        Settings::default()
    });

    let trapdoors = Trapdoors::load().unwrap_or_else(|error| {
        error!("Error loading trapdoors: {error}");
        Trapdoors::default()
    });

    let token = settings.discord_token.clone();
    let channel = settings.discord_channel;
    let address = settings.server_address.clone();
    let account = if settings.online_mode {
        Account::microsoft(&settings.account_username).await?
    } else {
        Account::offline(&settings.account_username)
    };

    settings.save()?;
    let mut client = SwarmBuilder::new()
        .set_swarm_handler(swarm_handler)
        .add_account(account)
        .add_plugins((settings, trapdoors))
        .add_plugins((ShaysPluginGroup, MinecraftCommandsPlugin));

    if !token.is_empty() {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let config = DiscordBotConfig::default()
            .gateway_intents(intents)
            .token(token);

        client = client
            .add_plugins(DiscordBotPlugin::new(config))
            .add_plugins(DiscordCommandsPlugin);

        if channel != ChannelId::default() {
            client = client.add_plugins(DiscordTrackerPlugin);
        }
    }

    client.start(address.as_str()).await?
}

/// # Errors
/// Will return `Err` if `Swarm::add_with_opts` fails.
pub async fn swarm_handler(mut swarm: Swarm, event: SwarmEvent, state: SwarmState) -> Result<()> {
    match event {
        SwarmEvent::Chat(chat_packet) => info!("{}", chat_packet.message().to_ansi()),
        SwarmEvent::Disconnect(account, options) => {
            swarm.add_with_opts(&account, state, &options).await?;
        }
        _ => {}
    }

    Ok(())
}

#[derive(Default)]
pub struct BoundedCounter<I>(I);

impl<I> Iterator for BoundedCounter<I>
where
    I: Copy + Bounded + One + AddAssign<I> + RemAssign<I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.0;

        self.0 %= I::max_value();
        self.0 += I::one();

        Some(i)
    }
}
