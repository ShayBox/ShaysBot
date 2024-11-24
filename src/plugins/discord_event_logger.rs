use azalea::{
    app::{App, Plugin, Update},
    blocks::Block,
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    packet_handling::game::PacketEvent,
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    TabList,
};
use bevy_discord::{http::DiscordHttpResource, runtime::tokio_runtime};
use serenity::{all::ChannelId, json::json};

use crate::{plugins::prelude::*, settings::BotSettings, BoundedCounter, CARGO_PKG_REPOSITORY};

pub struct DiscordEventLoggerPlugin;

impl Plugin for DiscordEventLoggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(GameTick, handle_check_for_updates)
            .add_systems(
                Update,
                (
                    handle_add_entity_packet,
                    handle_block_break_packet,
                    handle_block_update_packet,
                    handle_remove_entities_packet,
                    handle_player_info_remove_packet,
                    handle_player_info_update_packet,
                )
                    .after(handle_add_block_states)
                    .after(handle_add_player_profiles),
            );
    }
}

#[derive(Component, Default)]
pub struct UpdateCounter(BoundedCounter<u64>);

type InitQueryData = Entity;
type InitQueryFilter = (With<Player>, With<LocalEntity>, Without<UpdateCounter>);

type RunQueryData<'a> = (&'a mut UpdateCounter, &'a BotSettings);
type RunQueryFilter = (With<Player>, With<LocalEntity>, With<UpdateCounter>);

/// # Panics
/// Will panic if `shaysbot::check_for_updates` or `shaysbot::get_remote_version` fails.
pub fn handle_check_for_updates(
    /* Insert the update counter component */
    mut init_query: Query<InitQueryData, InitQueryFilter>,
    mut commands: Commands,
    /* Check for updates once a day */
    mut run_query: Query<RunQueryData, RunQueryFilter>,
    discord: Res<DiscordHttpResource>,
) {
    const DAY: u64 = 20 * 60 * 60 * 24;

    /* Insert the update counter component */
    for entity in &mut init_query {
        commands.entity(entity).insert(UpdateCounter::default());
    }

    /* Check for updates once a day */
    for (mut counter, bot_settings) in &mut run_query {
        let Some(ticks) = counter.0.next() else {
            return;
        };

        let channel_id = bot_settings.discord_channel;
        if channel_id == ChannelId::default() {
            return; /* Missing Channel ID */
        }

        if ticks % DAY == 0 && crate::check_for_updates().expect("Failed to check for updates") {
            let version = crate::get_remote_version().expect("Failed to check for updates");
            let client = discord.client();
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("An update is available: {CARGO_PKG_REPOSITORY}/releases/tag/{version}"),
                });

                if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });
        }
    }
}

fn handle_add_entity_packet(
    mut packet_events: EventReader<PacketEvent>,
    query: Query<(&TabList, &BotSettings)>,
    discord: Res<DiscordHttpResource>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
            continue;
        };

        if packet.entity_type != EntityKind::Player {
            continue;
        }

        let Ok((tab_list, bot_settings)) = query.get(event.entity) else {
            continue;
        };

        let Some((_, player_info)) = tab_list.iter().find(|(uuid, _)| uuid == &&packet.uuid) else {
            continue;
        };

        let channel_id = bot_settings.discord_channel;
        if channel_id == ChannelId::default() {
            return; /* Missing Channel ID */
        }

        let bot_name = bot_settings.account_username.clone();
        let player_name = player_info.profile.name.clone();
        let client = discord.client();
        tokio_runtime().spawn(async move {
            let map = json!({
                "content": format!("{player_name} has entered visual range of {bot_name}"),
            });

            if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                error!("{error}");
            };
        });
    }
}

fn handle_block_break_packet(
    mut packet_events: EventReader<PacketEvent>,
    query: Query<(&BlockStates, &BotSettings)>,
    discord: Res<DiscordHttpResource>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::BlockDestruction(packet) = event.packet.as_ref() else {
            continue;
        };

        let Ok((block_states, bot_settings)) = query.get(event.entity) else {
            continue;
        };

        let Some(block_state) = block_states.0.get(&packet.pos) else {
            continue;
        };

        let channel_id = bot_settings.discord_channel;
        if channel_id == ChannelId::default() {
            return; /* Missing Channel ID */
        }

        let block = Box::<dyn Block>::from(*block_state);
        if block.id().ends_with("shulker_box") {
            let block_name = block.as_registry_block();
            let bot_name = bot_settings.account_username.clone();
            let client = discord.client();
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{block_name:?} was mined in visual range of {bot_name}"),
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
    query: Query<&BotSettings>,
    discord: Res<DiscordHttpResource>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
            continue;
        };

        let Ok(bot_settings) = query.get(event.entity) else {
            continue;
        };

        let channel_id = bot_settings.discord_channel;
        if channel_id == ChannelId::default() {
            return; /* Missing Channel ID */
        }

        let block = Box::<dyn Block>::from(packet.block_state);
        if block.id().ends_with("shulker_box") {
            let block_name = block.as_registry_block();
            let bot_name = bot_settings.account_username.clone();
            let client = discord.client();
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{block_name:?} was placed in visual range of {bot_name}"),
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
    query: Query<(&PlayerProfiles, &BotSettings)>,
    discord: Res<DiscordHttpResource>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
            continue;
        };

        for entity_id in &packet.entity_ids {
            let Ok((player_profiles, bot_settings)) = query.get(event.entity) else {
                continue;
            };

            let Some(player_profile) = player_profiles.0.get(entity_id) else {
                continue;
            };

            let channel_id = bot_settings.discord_channel;
            if channel_id == ChannelId::default() {
                return; /* Missing Channel ID */
            }

            let bot_name = bot_settings.account_username.clone();
            let player_name = player_profile.name.clone();
            let client = discord.client();
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{player_name} has exited visual range of {bot_name}"),
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
    query: Query<(&PlayerProfiles, &BotSettings)>,
    discord: Res<DiscordHttpResource>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::PlayerInfoRemove(packet) = event.packet.as_ref() else {
            continue;
        };

        let Ok((player_profiles, bot_settings)) = query.get(event.entity) else {
            continue;
        };

        for profile_uuid in &packet.profile_ids {
            let Some((_, player_profile)) = player_profiles
                .0
                .iter()
                .find(|(_, profile)| &profile.uuid == profile_uuid)
            else {
                continue;
            };

            let channel_id = bot_settings.discord_channel;
            if channel_id == ChannelId::default() {
                return; /* Missing Channel ID */
            }

            let bot_name = bot_settings.account_username.clone();
            let player_name = player_profile.name.clone();
            let client = discord.client();
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{player_name} logged out in visual range of {bot_name}"),
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
    query: Query<(&PlayerProfiles, &BotSettings)>,
    discord: Res<DiscordHttpResource>,
) {
    for event in packet_events.read().cloned() {
        let ClientboundGamePacket::PlayerInfoUpdate(packet) = event.packet.as_ref() else {
            continue;
        };

        if !packet.actions.add_player {
            continue;
        }

        let Ok((player_profiles, bot_settings)) = query.get(event.entity) else {
            continue;
        };

        let profiles = packet.entries.clone().into_iter().filter_map(|info| {
            player_profiles.0.iter().find(|(_, profile)| {
                profile.uuid == info.profile.uuid && info.display_name.is_some()
            })
        });

        let channel_id = bot_settings.discord_channel;
        if channel_id == ChannelId::default() {
            return; /* Missing Channel ID */
        }

        for (_, player_profile) in profiles {
            let bot_name = bot_settings.account_username.clone();
            let player_name = player_profile.name.clone();
            let client = discord.client();
            tokio_runtime().spawn(async move {
                let map = json!({
                    "content": format!("{player_name} joined in visual range of {bot_name}"),
                });

                if let Err(error) = client.send_message(channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });
        }
    }
}
