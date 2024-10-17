use std::collections::VecDeque;

use anyhow::Result;
use azalea::Client;

use super::{CommandHandler, CommandResponse};
use crate::State;

#[derive(Clone)]
pub struct Pearl;

#[async_trait]
impl CommandHandler for Pearl {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["!pearl", "!tp", "!teleport", "!pull", "!here", "!home"]
    }

    async fn execute(
        &self,
        _args: VecDeque<&str>,
        _client: &Client,
        state: State,
        username: &str,
    ) -> Result<CommandResponse> {
        state.pearl_tx.send(username.to_owned()).await?;

        Ok(CommandResponse::None)
    }
}
