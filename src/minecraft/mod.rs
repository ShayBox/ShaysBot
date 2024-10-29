use std::sync::Arc;

use azalea::{Account, ClientBuilder};
use derive_config::{DeriveTomlConfig, DeriveYamlConfig};
use parking_lot::RwLock;

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
mod auto_look;
mod auto_pearl;
mod auto_totem;
mod commands;
mod pearl_tracker;

/// # Create and start the Minecraft bot client
///
/// # Errors
/// Will return `Err` if `ClientBuilder::start` fails.
#[allow(clippy::future_not_send)]
pub async fn start() -> anyhow::Result<()> {
    let settings = Settings::load().unwrap_or_default();
    let trapdoors = Trapdoors::load().unwrap_or_default();
    let address = settings.server_address.clone();
    let account = if settings.online {
        Account::microsoft(&settings.username).await?
    } else {
        Account::offline(&settings.username)
    };

    settings.save()?;
    let settings = Arc::new(RwLock::new(settings));
    let trapdoors = Arc::new(RwLock::new(trapdoors));
    let client = ClientBuilder::new().add_plugins((
        CommandsPlugin,
        PearlCommandPlugin,
        AntiAfkPlugin,
        AutoEatPlugin,
        AutoLookPlugin,
        AutoPearlPlugin,
        AutoTotemPlugin,
        PearlTrackerPlugin,
        SettingsPlugin(settings),
        TrapdoorsPlugin(trapdoors),
    ));

    client.start(account, address).await?
}
