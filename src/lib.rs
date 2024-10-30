#[macro_use]
extern crate lazy_regex;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate tracing;

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
use serenity::prelude::*;
use url::Url;

use crate::{
    plugins::ShaysPluginGroup,
    settings::{Settings, SettingsPlugin},
    trapdoors::{Trapdoor, Trapdoors, TrapdoorsPlugin},
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
    let settings = Settings::load().unwrap_or_default();
    let trapdoors = Trapdoors::load().unwrap_or_default();
    let address = settings.server_address.clone();
    let token = settings.discord_token.clone();
    let account = if settings.online {
        Account::microsoft(&settings.username).await?
    } else {
        Account::offline(&settings.username)
    };

    settings.save()?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let config = DiscordBotConfig::default()
        .gateway_intents(intents)
        .token(token);

    let client = SwarmBuilder::new()
        .set_swarm_handler(swarm_handler)
        .add_account(account)
        .add_plugins((
            ShaysPluginGroup,
            SettingsPlugin(settings),
            TrapdoorsPlugin(trapdoors),
            DiscordBotPlugin::new(config),
        ));

    client.start(address).await?
}

/// # Errors
/// Will return `Err` if `Swarm::add_with_opts` fails.
pub async fn swarm_handler(
    mut swarm: Swarm,
    event: SwarmEvent,
    state: SwarmState,
) -> anyhow::Result<()> {
    match event {
        SwarmEvent::Chat(chat_packet) => println!("{}", chat_packet.message().to_ansi()),
        SwarmEvent::Disconnect(account, options) => {
            swarm.add_with_opts(&account, state, &options).await?;
        }
        _ => {}
    }

    Ok(())
}

#[derive(Default)]
pub struct BoundedCounter<T>(T);

impl<T> Iterator for BoundedCounter<T>
where
    T: Copy + Bounded + One + AddAssign<T> + RemAssign<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let ticks = self.0;

        self.0 %= T::max_value();
        self.0 += T::one();

        Some(ticks)
    }
}
