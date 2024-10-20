use anyhow::Result;
use azalea::Client;

use super::EventHandler;
use crate::{plugins::prelude::*, State};

#[derive(Clone)]
pub struct Init;

#[async_trait]
impl EventHandler for Init {
    /// # Handle Login Events
    ///
    /// # Errors
    /// Will not return `Err`.
    async fn execute(self, client: Client, _state: State) -> Result<()> {
        client.init_anti_afk();

        Ok(())
    }
}
