pub mod prelude;

mod chat;
mod disconnect;
mod init;
mod packet;

use anyhow::Result;
use azalea::{Client, Event};

use crate::{events::prelude::*, State};

#[clonable]
#[async_trait]
pub trait EventHandler: Clone {
    async fn execute(self, client: Client, state: State) -> Result<()>;
}

impl State {
    /// # Event Handler
    ///
    /// # Errors
    /// Will not return `Err` because it is silently ignored by Azalea.
    ///
    /// # Panics
    /// Will panic if an event handler fails, to prevent silent errors.
    pub(crate) async fn handler(client: Client, event: Event, state: Self) -> Result<()> {
        match event {
            Event::Chat(packet) => Chat(packet).execute(client, state).await,
            Event::Disconnect(reason) => Disconnect(reason).execute(client, state).await,
            Event::Init => Init.execute(client, state).await,
            Event::Packet(packet) => Packet(packet).execute(client, state).await,

            _ => return Ok(()),
        }
        .expect("Failed to handle event");

        Ok(())
    }
}
