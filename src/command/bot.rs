use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{chat::ChatPacket, Client};

use crate::{ncr::NCReply, Message, State};

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
        ncr: Option<NCReply>,
    ) -> Result<()> {
        state
            .mc_queue
            .lock()
            .unwrap()
            .push(("[iambot]".into(), ncr));

        Ok(())
    }
}
