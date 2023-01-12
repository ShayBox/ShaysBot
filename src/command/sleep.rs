use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{BlockPos, ChatPacket, Client, Vec3};
use azalea_protocol::packets::game::{
    serverbound_interact_packet::InteractionHand,
    serverbound_use_item_on_packet::{BlockHitResult, ServerboundUseItemOnPacket},
};
use rand::prelude::*;

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
        let messages = vec![
            "I could use some sleep, I'll turn in for the night. Zzz",
            "I think I'll get some sleep now. Zzz",
            "I think I'll turn in and get some rest. Zzz",
            "I think I'll turn in for the night and get some sleep. Zzz",
            "I'm a bit tired now, I think I'll turn in. Zzz",
            "I'm feeling a bit fatigued, I think I'll turn in for the night. Zzz",
            "I'm feeling a bit wiped out, I think it's time to hit the hay. Zzz",
            "I'm feeling ready to turn in for the night. Zzz",
            "I'm feeling sleepier by the minute, I think it's time to call it a night. Zzz",
            "I'm feeling sleepy and ready to go to bed. Zzz",
            "I'm feeling tired and ready to drift off to sleep. Zzz",
            "I'm feeling tired and ready to go to sleep. Zzz",
            "I'm feeling tired and ready to sleep. Zzz",
            "I'm feeling tired and ready to turn in for the night. Zzz",
            "I'm getting ready to drift off now. Zzz",
            "I'm going to catch some z's now. Zzz",
            "I'm going to catch some z's. Zzz",
            "I'm going to get some rest now. Zzz",
            "I'm going to get some shut-eye. Zzz",
            "I'm going to get some sleep now. Zzz",
            "I'm going to head to bed and get some sleep. Zzz",
            "I'm going to head to bed now. Zzz",
            "I'm going to hit the hay. Zzz",
            "I'm going to tuck myself in and get some rest. Zzz",
            "I'm going to tuck myself in now. Zzz",
            "I'm going to turn in for the night. Zzz",
            "I'm off to bed now. Zzz",
            "I'm off to get some rest now. Zzz",
            "I'm ready to call it a night and get some sleep. Zzz",
            "I'm ready to drift off to dreamland now. Zzz",
            "I'm ready to drift off to dreamland. Zzz",
            "I'm ready to get some shut-eye. Zzz",
            "I'm ready to go to bed and get some rest. Zzz",
            "I'm ready to head to bed now. Zzz",
            "I'm starting to feel the weight of the day, I think it's time to get some rest. Zzz",
            "I'm starting to get drowsy, I think it's time to tuck myself in. Zzz",
            "I'm tired, I think it's time for me to hit the hay. Zzz",
            "It's been a long day, I'm ready to catch some shut-eye. Zzz",
            "It's time for me to catch some shut-eye. Zzz",
            "It's time for me to drift off to sleep. Zzz",
            "It's time for me to get some rest. Zzz",
            "It's time for me to turn in and get some rest. Zzz",
            "Ok, I'll get some rest now. Zzz",
            "Pro Tip: You can filter Zzz to hide *most* sleep messages. *cough*",
        ];

        let Some(message) = messages.choose(&mut thread_rng()) else {
            return Ok(())
        };

        state.mc_queue.lock().unwrap().push(message.to_string());

        let bed_pos = state.config.lock().unwrap().bed;
        let sleep_packet = ServerboundUseItemOnPacket {
            hand: InteractionHand::MainHand,
            block_hit: BlockHitResult {
                block_pos: BlockPos {
                    x: bed_pos.x,
                    y: bed_pos.y,
                    z: bed_pos.z,
                },
                direction: Default::default(),
                location: Vec3 {
                    x: bed_pos.x as f64,
                    y: bed_pos.y as f64,
                    z: bed_pos.z as f64,
                },
                inside: false,
            },
            sequence: 0,
        };

        client.write_packet(sleep_packet.get()).await?;

        Ok(())
    }
}
