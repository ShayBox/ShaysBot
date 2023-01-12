use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{ChatPacket, Client};

use crate::{Message, State};

#[derive(Clone)]
pub struct Command;

#[async_trait]
impl Message for Command {
    async fn message(
        &self,
        _client: Client,
        _chat: ChatPacket,
        state: State,
        _args: VecDeque<&str>,
    ) -> Result<()> {
        let commands = state
            .commands
            .keys()
            .map(|names| names.join("/"))
            .collect::<Vec<_>>()
            .join(", ");
        let message = format!("Commands: {commands}");
        state.mc_queue.lock().unwrap().push(message);

        Ok(())
    }
}
