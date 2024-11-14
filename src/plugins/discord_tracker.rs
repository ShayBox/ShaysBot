use std::collections::HashMap;

use azalea::{
    app::{App, Plugin, Update},
    auth::game_profile::GameProfile,
    blocks::{Block, BlockState},
    ecs::prelude::*,
    packet_handling::game::PacketEvent,
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    BlockPos,
    TabList,
};
use bevy_discord::{http::DiscordHttpResource, runtime::tokio_runtime};
use serenity::json::json;

use crate::Settings;

pub struct DiscordTrackerPlugin;

impl Plugin for DiscordTrackerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockStates::default())
            .insert_resource(PlayerProfiles::default())
            .add_systems(
                Update,
                (
                    handle_add_entity_packet,
                    handle_block_break_packet,
                    handle_block_update_packet,
                    handle_remove_entities_packet,
                ),
            );
    }
}

#[derive(Default, Resource)]
pub struct BlockStates(HashMap<BlockPos, BlockState>);

#[derive(Default, Resource)]
pub struct PlayerProfiles(HashMap<u32, GameProfile>);

fn handle_add_entity_packet(
    mut packet_events: EventReader<PacketEvent>,
    mut player_profiles: ResMut<PlayerProfiles>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
    query: Query<&TabList>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
            continue;
        };

        if packet.entity_type != EntityKind::Player {
            continue;
        }

        let Ok(tab_list) = query.get(event.entity) else {
            continue;
        };

        let Some((_, info)) = tab_list.iter().find(|(uuid, _)| uuid == &&packet.uuid) else {
            continue;
        };

        let profile = info.profile.clone();
        let username = profile.name.clone();
        player_profiles.0.insert(packet.data, profile);

        let client = discord.client();
        let channel_id = settings.discord_channel;
        tokio_runtime().spawn(async move {
            let map = json!({
                "content": format!("{username} has entered visual range"),
            });

            if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                error!("{error}");
            };
        });
    }
}

fn handle_block_break_packet(
    mut packet_events: EventReader<PacketEvent>,
    block_states: Res<BlockStates>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::BlockDestruction(packet) = event.packet.as_ref() else {
            continue;
        };

        let Some(block_state) = block_states.0.get(&packet.pos) else {
            continue;
        };

        let block = Box::<dyn Block>::from(*block_state);
        let block_id = block.id();
        if block_id.ends_with("shulker_box") {
            let client = discord.client();
            let channel_id = settings.discord_channel;
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{block_id} was mined in visual range"),
                });

                if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });
        }
    }
}

fn handle_block_update_packet(
    mut packet_events: EventReader<PacketEvent>,
    mut block_states: ResMut<BlockStates>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
            continue;
        };

        block_states.0.insert(packet.pos, packet.block_state);

        let block = Box::<dyn Block>::from(packet.block_state);
        let block_id = block.id();
        if block_id.ends_with("shulker_box") {
            let client = discord.client();
            let channel_id = settings.discord_channel;
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{block_id} was placed in visual range"),
                });

                if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });
        }
    }
}

fn handle_remove_entities_packet(
    mut packet_events: EventReader<PacketEvent>,
    mut player_profiles: ResMut<PlayerProfiles>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
            continue;
        };

        for entity_id in &packet.entity_ids {
            let Some(profile) = player_profiles.0.remove(entity_id) else {
                continue;
            };

            let client = discord.client();
            let username = profile.name.clone();
            let channel_id = settings.discord_channel;
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{username} has exited visual range"),
                });

                if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });
        }
    }
}