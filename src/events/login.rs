use anyhow::Result;
use azalea::{
    core::direction::Direction,
    packet_handling::game::SendPacketEvent,
    pathfinder::{goals::ReachBlockPosGoal, PathfinderClientExt},
    protocol::packets::game::{
        serverbound_interact_packet::InteractionHand,
        serverbound_use_item_on_packet::{BlockHit, ServerboundUseItemOnPacket},
        ServerboundGamePacket,
    },
    BlockPos,
    Client,
    Vec3,
};

use super::EventHandler;
use crate::State;

#[derive(Clone)]
pub struct Login;

#[async_trait]
impl EventHandler for Login {
    /// # Handle Login Events
    ///
    /// # Errors
    /// Will not return `Err`.
    async fn execute(self, client: Client, state: State) -> Result<()> {
        #[allow(clippy::significant_drop_in_scrutinee)] /* Should always be locked */
        while let Some(username) = state.pearl_rx.lock().await.recv().await {
            let Some(uuid) = client
                .tab_list()
                .into_iter()
                .find(|(_, info)| info.profile.name == username)
                .map(|(uuid, _)| uuid)
            else {
                debug!("Failed to find {username} in tab list");
                return Ok(());
            };

            let trapdoors = state.trapdoors.read().await.0.clone();
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
                if !state.settings.read().await.quiet {
                    let command = format!("w {username} [404] Pearl not found.");
                    client.send_command_packet(&command);
                }

                return Ok(());
            };

            if !state.settings.read().await.quiet {
                let command = format!("w {username} [202] I'm on my way!");
                client.send_command_packet(&command);
            }

            state.wait_for_pathfinder(&client).await?;
            client.goto(ReachBlockPosGoal {
                chunk_storage: client.world().read().chunks.clone(),
                pos:           trapdoor.block_pos,
            });

            state.wait_for_pathfinder(&client).await?;
            client.ecs.lock().send_event(SendPacketEvent {
                entity: client.entity,
                packet: ServerboundGamePacket::UseItemOn(ServerboundUseItemOnPacket {
                    hand:      InteractionHand::MainHand,
                    sequence:  0,
                    block_hit: BlockHit {
                        block_pos: trapdoor.block_pos,
                        direction: Direction::Down,
                        inside:    true,
                        location:  Vec3 {
                            x: f64::from(trapdoor.block_pos.x) + 0.5,
                            y: f64::from(trapdoor.block_pos.y) + 0.5,
                            z: f64::from(trapdoor.block_pos.z) + 0.5,
                        },
                    },
                }),
            });

            if !state.settings.read().await.quiet {
                let command = format!("w {username} [200] OK");
                client.send_command_packet(&command);
            }
        }

        Ok(())
    }
}
