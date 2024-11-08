#![feature(trivial_bounds)]

extern crate core;
#[macro_use]
extern crate lazy_regex;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate structstruck;
#[macro_use]
extern crate tracing;

pub mod ncr;
pub mod plugins;
pub mod settings;
pub mod trapdoors;

use std::ops::{AddAssign, RemAssign};

use azalea::{
    ecs::prelude::*,
    prelude::*,
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
    Account,
};
use bevy_discord::bot::{DiscordBotConfig, DiscordBotPlugin};
use derive_config::{DeriveTomlConfig, DeriveYamlConfig};
use num_traits::{Bounded, One};
use plugins::prelude::DiscordCommandsPlugin;
use serenity::prelude::*;
use url::Url;

pub use crate::{
    plugins::ShaysPluginGroup,
    settings::Settings,
    trapdoors::{Trapdoor, Trapdoors},
};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CARGO_PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// # Check for updates using GitHub's latest release link redirect
///
/// # Errors
/// Will return `Err` if `ureq::get` fails.
pub fn check_for_updates() -> anyhow::Result<bool> {
    let response = ureq::get(CARGO_PKG_HOMEPAGE).call()?;

    if let Ok(parsed_url) = Url::parse(response.get_url()) {
        if let Some(segments) = parsed_url.path_segments() {
            if let Some(remote_version) = segments.last() {
                return Ok(remote_version > CARGO_PKG_VERSION);
            }
        }
    }

    Ok(false)
}

#[derive(Clone, Component, Default, Resource)]
pub struct SwarmState;

/// # Create and start the Minecraft bot client
///
/// # Errors
/// Will return `Err` if `ClientBuilder::start` fails.
#[allow(clippy::future_not_send)]
pub async fn start() -> anyhow::Result<()> {
    let settings = Settings::load().unwrap_or_else(|error| {
        eprintln!("Error loading settings: {error}");
        Settings::default()
    });

    let trapdoors = Trapdoors::load().unwrap_or_else(|error| {
        eprintln!("Error loading trapdoors: {error}");
        Trapdoors::default()
    });

    let address = settings.server_address.clone();
    let token = settings.discord_token.clone();
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
        .add_plugins(ShaysPluginGroup);

    if !token.is_empty() {
        let config = DiscordBotConfig::default()
            .gateway_intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
            .token(token);

        client = client.add_plugins((DiscordBotPlugin::new(config), DiscordCommandsPlugin));
    }

    client.start(address.as_str()).await?
}

/// # Errors
/// Will return `Err` if `Swarm::add_with_opts` fails.
pub async fn swarm_handler(
    mut swarm: Swarm,
    event: SwarmEvent,
    state: SwarmState,
) -> anyhow::Result<()> {
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
