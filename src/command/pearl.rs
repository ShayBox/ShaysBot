use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{chat::ChatPacket, BlockPos, Client, Vec3};
use azalea_protocol::packets::game::{
    serverbound_interact_packet::InteractionHand,
    serverbound_use_item_on_packet::{BlockHitResult, ServerboundUseItemOnPacket},
};

use crate::{ncr::NCREncryption, Message, State};

#[derive(Clone)]
pub struct Command;

#[async_trait]
impl Message for Command {
    async fn message(
        &self,
        client: Client,
        chat: ChatPacket,
        state: State,
        _args: VecDeque<&str>,
        ncr: Option<NCREncryption>,
    ) -> Result<()> {
        let Some(mut username) = chat.username() else {
            return Ok(());
        };

        // Strip prefixes
        let prefixes = username.split(' ').collect::<Vec<_>>();
        if let Some(last) = prefixes.last() {
            username = last.to_string();
        }

        if username == "ShayBox" {
            state
                .mc_queue
                .lock()
                .unwrap()
                .push(("Teleporting...".into(), ncr));

            let pearl_pos = state.config.lock().unwrap().pearl;
            let sleep_packet = ServerboundUseItemOnPacket {
                hand: InteractionHand::MainHand,
                block_hit: BlockHitResult {
                    block_pos: BlockPos {
                        x: pearl_pos.x,
                        y: pearl_pos.y,
                        z: pearl_pos.z,
                    },
                    direction: Default::default(),
                    location: Vec3 {
                        x: pearl_pos.x as f64,
                        y: pearl_pos.y as f64,
                        z: pearl_pos.z as f64,
                    },
                    inside: false,
                },
                sequence: 0,
            };

            client.write_packet(sleep_packet.get());
        } else {
            let message = "You do not have permission to use this command.";
            state.mc_queue.lock().unwrap().push((message.into(), ncr));
        }

        Ok(())
    }
}
