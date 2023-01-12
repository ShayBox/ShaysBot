use std::{collections::VecDeque, ops::Deref};

use anyhow::Result;
use async_trait::async_trait;
use azalea::{ping::ping_server, ChatPacket, Client};
use dotenvy_macro::dotenv;

use crate::{Message, State};

#[derive(Clone)]
pub struct Command;

#[async_trait]
impl Message for Command {
    async fn message(
        &self,
        client: Client,
        _chat: ChatPacket,
        state: State,
        _args: VecDeque<&str>,
    ) -> Result<()> {
        let response = ping_server(dotenv!("ADDRESS")).await?;
        let online = response.players.online;
        let max = response.players.max;

        let players = {
            let config = state.config.lock().unwrap();
            client
                .players
                .read()
                .deref()
                .iter()
                .map(|(_uuid, info)| info.profile.name.to_owned())
                .filter(|name| !config.bots.contains(name))
                .collect::<Vec<_>>()
        };
        let filtered = players.len();
        let player_list = players.join(", ");

        let message = format!("[{filtered}/{online}/{max}]: {player_list}");
        state.mc_queue.lock().unwrap().push(message);

        Ok(())
    }
}
