use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{chat::ChatPacket, Client};
use dyn_clonable::*;

use crate::{ncr::NCReply, State};

#[clonable]
#[async_trait]
pub trait Message: Clone {
    async fn message(
        &self,
        client: Client,
        chat: ChatPacket,
        state: State,
        args: VecDeque<&str>,
        ncr: Option<NCReply>,
    ) -> Result<()>;
}
