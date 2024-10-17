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
mod settings;
mod trapdoor;

use std::sync::{Arc, LazyLock};

use azalea::prelude::*;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
    RwLock,
};

use crate::{
    commands::{prelude::*, CommandHandler},
    events::{
        prelude::{Disconnect, *},
        EventHandler,
    },
};
pub use crate::{
    settings::Settings,
    trapdoor::{Trapdoor, Trapdoors},
};

type CMD = Box<dyn CommandHandler + Send + Sync>;

macro_rules! cmd {
    ($x:expr) => {
        Box::new($x) as CMD
    };
}

static COMMANDS: LazyLock<[CMD; 1]> = LazyLock::new(|| [cmd!(Pearl)]);

#[derive(Clone, Component, Resource)]
pub struct State {
    pearl_tx:  Sender<String>,
    pearl_rx:  Arc<Mutex<Receiver<String>>>,
    settings:  Arc<RwLock<Settings>>,
    trapdoors: Arc<RwLock<Trapdoors>>,
}

impl Default for State {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        Self {
            pearl_tx:  tx,
            pearl_rx:  Arc::new(Mutex::new(rx)),
            settings:  Arc::new(RwLock::default()),
            trapdoors: Arc::new(RwLock::default()),
        }
    }
}

impl State {
    #[must_use]
    pub fn new(settings: Settings, trapdoors: Trapdoors) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        Self {
            pearl_tx:  tx,
            pearl_rx:  Arc::new(Mutex::new(rx)),
            settings:  Arc::new(RwLock::new(settings)),
            trapdoors: Arc::new(RwLock::new(trapdoors)),
        }
    }

    /// # Create and start the Minecraft bot client
    ///             
    /// # Errors
    /// Will return `Err` if `ClientBuilder::start` fails.
    #[allow(clippy::future_not_send)]
    pub async fn start_minecraft(self) -> anyhow::Result<()> {
        let config = self.settings.read().await.clone();
        let account = if config.online {
            Account::microsoft(&config.username).await?
        } else {
            Account::offline(&config.username)
        };

        let client = ClientBuilder::new()
            .set_handler(Self::handler)
            .set_state(self);

        client.start(account, config.server_address).await?
    }

    /// # Event Handler
    ///
    /// # Errors
    /// Will not return `Err` because it is silently ignored by Azalea.
    ///
    /// # Panics
    /// Will panic if an event handler fails, to prevent silent errors.
    async fn handler(client: Client, event: Event, state: Self) -> anyhow::Result<()> {
        match event {
            Event::Chat(packet) => Chat(packet).execute(client, state).await,
            Event::Disconnect(reason) => Disconnect(reason).execute(client, state).await,
            Event::Packet(packet) => Packet(packet).execute(client, state).await,
            Event::Login => Login.execute(client, state).await,

            _ => return Ok(()),
        }
        .expect("Failed to handle event");

        Ok(())
    }
}
