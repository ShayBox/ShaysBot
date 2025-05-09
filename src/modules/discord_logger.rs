use azalea::{
    app::{App, Plugin, Update},
    blocks::Block,
    disconnect::DisconnectEvent,
    ecs::prelude::*,
    packet::{game::ReceiveGamePacketEvent, login::ReceiveLoginPacketEvent},
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    GameProfileComponent,
    TabList,
};
use bevy_discord::res::DiscordHttpResource;
use serenity::{
    all::{ChannelId, Http},
    json::json,
};

use crate::prelude::*;

/// Log events such as `Visual Range` to Discord
pub struct DiscordLoggerPlugin;

impl Plugin for DiscordLoggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            Self::handle_check_for_updates.after(GameTickPlugin::handle_game_ticks),
        )
        .add_systems(
            Update,
            (
                Self::handle_add_entity_packets,
                Self::handle_block_break_packets,
                Self::handle_block_update_packets,
                Self::handle_disconnect_events.after(AutoLeavePlugin::handle_add_entity_packets),
                Self::handle_login_packets,
                Self::handle_player_info_remove_packets,
                Self::handle_player_info_update_packets,
                Self::handle_remove_entities_packets,
            )
                .after(BlockStatePlugin::handle_block_update_packets)
                .after(PlayerProfilePlugin::handle_add_entity_packets),
        );
    }
}

impl DiscordLoggerPlugin {
    /// # Panics
    /// Will panic if `shaysbot::check_for_updates` or `shaysbot::get_remote_version` fails.
    pub fn handle_check_for_updates(
        mut query: Query<(&GameTicks, &LocalSettings)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        const DAY_DELAY: u128 = 20 * 60 * 60 * 24;
        let Some(discord) = discord else {
            return;
        };

        for (game_ticks, bot_settings) in &mut query {
            let channel_id = bot_settings.discord_channel;
            if channel_id == ChannelId::default() {
                return; /* Missing Channel ID */
            }

            if game_ticks.0 % DAY_DELAY != 0 {
                continue;
            }

            if check_for_updates().expect("Failed to check for updates") {
                let version = get_remote_version().expect("Failed to check for updates");
                let content = format!("Update: {CARGO_PKG_REPOSITORY}/releases/tag/{version}");
                tokio::spawn(send_message(content, channel_id, discord.client()));
            }
        }
    }

    fn handle_add_entity_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        query: Query<(&TabList, &LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
                continue;
            };

            if packet.entity_type != EntityKind::Player {
                continue;
            }

            let Ok((tab_list, local_settings, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let Some((_, player_info)) = tab_list.iter().find(|(uuid, _)| uuid == &&packet.uuid)
            else {
                continue;
            };

            let channel_id = local_settings.discord_channel;
            if channel_id == ChannelId::default() {
                return; /* Missing Channel ID */
            }

            let player_name = player_info.profile.name.clone();
            let username = game_profile.name.clone();
            let content = format!("[{username}] {player_name} has entered visual range.");
            tokio::spawn(send_message(content, channel_id, discord.client()));
        }
    }

    fn handle_block_break_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        query: Query<(&BlockStates, &LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::BlockDestruction(packet) = event.packet.as_ref() else {
                continue;
            };

            let Ok((block_states, local_settings, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let Some(block_state) = block_states.0.get(&packet.pos) else {
                continue;
            };

            let channel_id = local_settings.discord_channel;
            if channel_id == ChannelId::default() {
                return; /* Missing Channel ID */
            }

            let block = Box::<dyn Block>::from(*block_state);
            if block.id().ends_with("shulker_box") {
                let block_name = block.as_registry_block();
                let username = game_profile.name.clone();
                let content = format!("[{username}] {block_name:?} was mined in visual range.");
                tokio::spawn(send_message(content, channel_id, discord.client()));
            }
        }
    }

    fn handle_block_update_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        query: Query<(&LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
                continue;
            };

            let Ok((local_settings, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let channel_id = local_settings.discord_channel;
            if channel_id == ChannelId::default() {
                return; /* Missing Channel ID */
            }

            let block = Box::<dyn Block>::from(packet.block_state);
            if block.id().ends_with("shulker_box") {
                let block_name = block.as_registry_block();
                let username = game_profile.name.clone();
                let content = format!("[{username}] {block_name:?} was placed in visual range.");
                tokio::spawn(send_message(content, channel_id, discord.client()));
            }
        }
    }

    pub fn handle_disconnect_events(
        mut events: EventReader<DisconnectEvent>,
        query: Query<(&LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in events.read() {
            let Ok((local_settings, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let channel_id = local_settings.discord_channel;
            if channel_id == ChannelId::default() {
                continue; /* Missing Channel ID */
            }

            let Some(reason) = event.reason.clone() else {
                continue; /* Missing Reason */
            };

            let username = game_profile.name.clone();
            let content = if str!(reason).starts_with(ZENITH_PREFIX) {
                format!("[{username}] Disabling AutoReconnect")
            } else {
                format!("[{username}] left the game. | {reason}")
            };
            tokio::spawn(send_message(content, channel_id, discord.client()));
        }
    }

    fn handle_login_packets(
        mut events: EventReader<ReceiveLoginPacketEvent>,
        query: Query<(&LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in events.read() {
            let Ok((local_settings, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let channel_id = local_settings.discord_channel;
            if channel_id == ChannelId::default() {
                continue; /* Missing Channel ID */
            }

            let username = &game_profile.name;
            let content = format!("[{username}] joined the game.");
            tokio::spawn(send_message(content, channel_id, discord.client()));
        }
    }

    fn handle_player_info_remove_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        query: Query<(&PlayerProfiles, &LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::PlayerInfoRemove(packet) = event.packet.as_ref() else {
                continue;
            };

            let Ok((player_profiles, local_settings, game_profile)) = query.get(event.entity)
            else {
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

                let channel_id = local_settings.discord_channel;
                if channel_id == ChannelId::default() {
                    return; /* Missing Channel ID */
                }

                let player_name = player_profile.name.clone();
                let username = game_profile.name.clone();
                let content = format!("[{username}] {player_name} logged out in visual range.");
                tokio::spawn(send_message(content, channel_id, discord.client()));
            }
        }
    }

    fn handle_player_info_update_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        query: Query<(&PlayerProfiles, &LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in packet_events.read().cloned() {
            let ClientboundGamePacket::PlayerInfoUpdate(packet) = event.packet.as_ref() else {
                continue;
            };

            if !packet.actions.add_player {
                continue;
            }

            let Ok((player_profiles, local_settings, game_profile)) = query.get(event.entity)
            else {
                continue;
            };

            let profiles = packet.entries.clone().into_iter().filter_map(|info| {
                player_profiles.0.iter().find(|(_, profile)| {
                    profile.uuid == info.profile.uuid && info.display_name.is_some()
                })
            });

            let channel_id = local_settings.discord_channel;
            if channel_id == ChannelId::default() {
                return; /* Missing Channel ID */
            }

            for (_, player_profile) in profiles {
                let player_name = player_profile.name.clone();
                let username = game_profile.name.clone();
                let content = format!("[{username}] {player_name} joined in visual range.");
                tokio::spawn(send_message(content, channel_id, discord.client()));
            }
        }
    }

    fn handle_remove_entities_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        query: Query<(&PlayerProfiles, &LocalSettings, &GameProfileComponent)>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
                continue;
            };

            for entity_id in &packet.entity_ids {
                let Ok((player_profiles, local_settings, game_profile)) = query.get(event.entity)
                else {
                    continue;
                };

                let Some(player_profile) = player_profiles.0.get(entity_id) else {
                    continue;
                };

                let channel_id = local_settings.discord_channel;
                if channel_id == ChannelId::default() {
                    return; /* Missing Channel ID */
                }

                let player_name = player_profile.name.clone();
                let username = game_profile.name.clone();
                let content = format!("[{username}] {player_name} has exited visual range.");
                tokio::spawn(send_message(content, channel_id, discord.client()));
            }
        }
    }
}

pub async fn send_message(content: impl ToString, channel_id: ChannelId, client: Arc<Http>) {
    let map = json!({
        "content": str!(content),
    });

    if let Err(error) = client.send_message(channel_id, vec![], &map).await {
        error!("{error}");
    };
}
