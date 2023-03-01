use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{chat::ChatPacket, ping::ping_server, Client};
use dotenvy_macro::dotenv;

use crate::{ncr::NCReply, Message, State};

#[derive(Clone)]
pub struct Command;

#[async_trait]
impl Message for Command {
    async fn message(
        &self,
        mut client: Client,
        _chat: ChatPacket,
        state: State,
        _args: VecDeque<&str>,
        ncr: Option<NCReply>,
    ) -> Result<()> {
        let response = ping_server(dotenv!("ADDRESS")).await?;
        let online = response.players.online;
        let max = response.players.max;

        let players = {
            let config = state.config.lock().unwrap();
            client
                .players()
                .values()
                .map(|info| info.profile.name.to_owned())
                .filter(|name| !config.bots.contains(name))
                .collect::<Vec<_>>()
        };
        let filtered = players.len();
        let player_list = players.join(", ");

        let message = format!("[{filtered}/{online}/{max}]: {player_list}");
        state.mc_queue.lock().unwrap().push((message, ncr));

        Ok(())
    }
}
