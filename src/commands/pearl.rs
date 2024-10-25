use std::collections::VecDeque;

use anyhow::Result;
use azalea::{prelude::*, BlockPos};

use super::{CommandHandler, CommandResponse};
use crate::{plugins::prelude::AutoPearlClientExt, State};

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
        client: &Client,
        state: &State,
        username: &str,
    ) -> Result<CommandResponse> {
        let Some(uuid) = client
            .tab_list()
            .into_iter()
            .find(|(_, info)| info.profile.name == username)
            .map(|(uuid, _)| uuid)
        else {
            let message = format!("Failed to find {username} in tab list");
            return Ok(CommandResponse::Whisper(message));
        };

        let trapdoors = state.trapdoors.read().0.clone();
        let Some(trapdoor) = trapdoors
            .clone()
            .into_values()
            .filter(|trapdoor| trapdoor.owner_uuid == uuid)
            .min_by_key(|trapdoor| {
                let shared_count = trapdoors
                    .values()
                    .filter(|td| td.block_pos == trapdoor.block_pos)
                    .filter(|td| td.owner_uuid != trapdoor.owner_uuid)
                    .count();

                let client_pos = BlockPos::from(client.position());
                let distance = (client_pos.x - trapdoor.block_pos.x).abs()
                    + (client_pos.y - trapdoor.block_pos.y).abs()
                    + (client_pos.z - trapdoor.block_pos.z).abs();

                // First compare by shared count, then by distance
                (shared_count, distance)
            })
        else {
            let message = String::from("[404] Pearl not found.");
            return Ok(CommandResponse::Whisper(message));
        };

        client.pearl(trapdoor.block_pos);
        if !state.settings.read().quiet {
            let command = format!("w {username} [202] I'm on my way!");
            client.send_command_packet(&command);
        }

        state.wait_for_pathfinder(client).await?;

        let message = String::from("[200] OK");
        Ok(CommandResponse::Whisper(message))
    }
}
