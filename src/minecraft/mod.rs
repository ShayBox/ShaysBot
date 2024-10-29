use anyhow::Result;
use azalea::{
    ecs::prelude::*,
    prelude::*,
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
    Account,
};
use derive_config::{DeriveTomlConfig, DeriveYamlConfig};

use crate::{
    minecraft::{commands::CommandsPlugin, prelude::*},
    settings::SettingsPlugin,
    trapdoors::TrapdoorsPlugin,
    Settings,
    Trapdoors,
};

pub mod prelude;

mod anti_afk;
mod auto_eat;
mod auto_exit;
mod auto_look;
mod auto_pearl;
mod auto_totem;
mod commands;
mod pearl_tracker;

#[derive(Clone, Component, Default, Resource)]
pub struct SwarmState;

/// # Create and start the Minecraft bot client
///
/// # Errors
/// Will return `Err` if `ClientBuilder::start` fails.
#[allow(clippy::future_not_send)]
pub async fn start() -> Result<()> {
    let settings = Settings::load().unwrap_or_default();
    let trapdoors = Trapdoors::load().unwrap_or_default();
    let address = settings.server_address.clone();
    let account = if settings.online {
        Account::microsoft(&settings.username).await?
    } else {
        Account::offline(&settings.username)
    };

    settings.save()?;
    let client = SwarmBuilder::new()
        .set_swarm_handler(swarm_handler)
        .add_account(account)
        .add_plugins((
            CommandsPlugin,
            PearlCommandPlugin,
            PlaytimeCommandPlugin,
            SeenCommandPlugin,
        ))
        .add_plugins((
            AntiAfkPlugin,
            AutoEatPlugin,
            AutoExitPlugin,
            AutoLookPlugin,
            AutoPearlPlugin,
            AutoTotemPlugin,
            PearlTrackerPlugin,
            SettingsPlugin(settings),
            TrapdoorsPlugin(trapdoors),
        ));

    client.start(address).await?
}

/// # Errors
/// Will return `Err` if `Swarm::add_with_opts` fails.
pub async fn swarm_handler(mut swarm: Swarm, event: SwarmEvent, state: SwarmState) -> Result<()> {
    match event {
        SwarmEvent::Chat(chat_packet) => println!("{}", chat_packet.message().to_ansi()),
        SwarmEvent::Disconnect(account, options) => {
            swarm.add_with_opts(&account, state, &options).await?;
        }
        _ => {}
    }

    Ok(())
}
