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
        let message = "Please use https://github.com/HKS-HNS/No-Chat-Reports with the 'AAA===' passphrase and 'AES/CFB8+MC256' algorithm for optimal space savings";
        state.mc_queue.lock().unwrap().push(message.into());

        Ok(())
    }
}
