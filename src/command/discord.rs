use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{chat::ChatPacket, Client};

use crate::{ncr::NCREncryption, Message, State};

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
        ncr: Option<NCREncryption>,
    ) -> Result<()> {
        let message = "My Discord bridge is available at https://discord.shaybox.com";
        state.mc_queue.lock().unwrap().push((message.into(), ncr));

        Ok(())
    }
}
