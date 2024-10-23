#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate dyn_clonable;
#[macro_use]
extern crate lazy_regex;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate tracing;

mod commands;
mod events;
mod packets;
mod plugins;
mod settings;
mod trapdoor;

use std::sync::Arc;

use anyhow::Result;
use azalea::prelude::*;
use tokio::sync::RwLock;

pub use crate::{
    plugins::ShaysPluginGroup,
    settings::Settings,
    trapdoor::{Trapdoor, Trapdoors},
};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CARGO_PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// # Check for updates using GitHub's latest release link redirect
///
/// # Errors
/// Will return `Err` if `reqwest::get` fails.
pub async fn check_for_updates() -> reqwest::Result<bool> {
    let response = reqwest::get(CARGO_PKG_HOMEPAGE).await?;
    if let Some(segments) = response.url().path_segments() {
        if let Some(remote_version) = segments.last() {
            return Ok(remote_version > CARGO_PKG_VERSION);
        };
    };

    Ok(false)
}

#[derive(Clone, Component, Resource)]
pub struct State {
    settings:  Arc<RwLock<Settings>>,
    trapdoors: Arc<RwLock<Trapdoors>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            settings:  Arc::new(RwLock::default()),
            trapdoors: Arc::new(RwLock::default()),
        }
    }
}

impl State {
    #[must_use]
    pub fn new(settings: Settings, trapdoors: Trapdoors) -> Self {
        Self {
            settings:  Arc::new(RwLock::new(settings)),
            trapdoors: Arc::new(RwLock::new(trapdoors)),
        }
    }

    /// # Create and start the Minecraft bot client
    ///
    /// # Errors
    /// Will return `Err` if `ClientBuilder::start` fails.
    #[allow(clippy::future_not_send)]
    pub async fn start(self) -> Result<()> {
        let settings = self.settings.read().await.clone();
        let account = if settings.online {
            Account::microsoft(&settings.username).await?
        } else {
            Account::offline(&settings.username)
        };

        let client = ClientBuilder::new()
            .add_plugins(ShaysPluginGroup)
            .set_handler(Self::handler)
            .set_state(self);

        client.start(account, settings.server_address).await?
    }
}
