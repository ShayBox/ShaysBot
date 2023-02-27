use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{chat::ChatPacket, Client};

use crate::{Message, State};

#[derive(Clone)]
pub struct Command;

#[async_trait]
impl Message for Command {
    async fn message(
        &self,
        _client: Client,
        chat: ChatPacket,
        state: State,
        _args: VecDeque<&str>,
    ) -> Result<()> {
        let Some(mut username) = chat.username() else {
            return Ok(())
        };

        // Strip prefixes
        let prefixes = username.split(' ').collect::<Vec<_>>();
        if let Some(last) = prefixes.last() {
            username = last.to_string();
        }

        let message = format!("Hello, {username}");
        state.mc_queue.lock().unwrap().push(message);

        Ok(())
    }
}
