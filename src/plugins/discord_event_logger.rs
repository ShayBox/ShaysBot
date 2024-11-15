use azalea::{
    app::{App, Plugin, Update},
    blocks::Block,
    ecs::prelude::*,
    packet_handling::game::PacketEvent,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    TabList,
};
use bevy_discord::{http::DiscordHttpResource, runtime::tokio_runtime};
use serenity::json::json;

use crate::{
    handle_add_player_profiles,
    plugins::block_state_tracker::BlockStates,
    PlayerProfiles,
    Settings,
};

pub struct DiscordEventLoggerPlugin;

impl Plugin for DiscordEventLoggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_add_entity_packet,
                handle_block_break_packet,
                handle_block_update_packet,
                handle_remove_entities_packet,
                handle_player_info_remove_packet,
                handle_player_info_update_packet.after(handle_add_player_profiles),
            ),
        );
    }
}

fn handle_add_entity_packet(
    mut packet_events: EventReader<PacketEvent>,
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

        let client = discord.client();
        let username = info.profile.name.clone();
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
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
            continue;
        };

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
    player_profiles: Res<PlayerProfiles>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
            continue;
        };

        for entity_id in &packet.entity_ids {
            let Some(profile) = player_profiles.0.get(entity_id) else {
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
fn handle_player_info_remove_packet(
    mut packet_events: EventReader<PacketEvent>,
    player_profiles: Res<PlayerProfiles>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::PlayerInfoRemove(packet) = event.packet.as_ref() else {
            continue;
        };

        for profile_uuid in &packet.profile_ids {
            let Some((_, profile)) = player_profiles
                .0
                .iter()
                .find(|(_, profile)| &profile.uuid == profile_uuid)
            else {
                continue;
            };

            let client = discord.client();
            let username = profile.name.clone();
            let channel_id = settings.discord_channel;
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{username} logged out in visual range"),
                });

                if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });
        }
    }
}
fn handle_player_info_update_packet(
    mut packet_events: EventReader<PacketEvent>,
    player_profiles: Res<PlayerProfiles>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in packet_events.read().cloned() {
        let ClientboundGamePacket::PlayerInfoUpdate(packet) = event.packet.as_ref() else {
            continue;
        };

        let profiles = packet.entries.clone().into_iter().filter_map(|info| {
            player_profiles.0.iter().find(|(_, profile)| {
                profile.uuid == info.profile.uuid && info.display_name.is_some()
            })
        });

        for (_, profile) in profiles {
            let client = discord.client();
            let username = profile.name.clone();
            let channel_id = settings.discord_channel;
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{username} joined in visual range"),
                });

                if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });
        }
    }
}
