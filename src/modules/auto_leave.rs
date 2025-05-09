use azalea::{
    app::{App, Plugin, PreUpdate, Update},
    auto_reconnect::start_rejoin_on_disconnect,
    connection::RawConnection,
    disconnect::DisconnectEvent,
    ecs::prelude::*,
    events::disconnect_listener,
    packet::game::{ReceiveGamePacketEvent, SendPacketEvent},
    protocol::packets::game::{ClientboundGamePacket, ServerboundGamePacket, ServerboundPong},
    registry::EntityKind,
    FormattedText,
    GameProfileComponent,
    TabList,
};
use itertools::Itertools;

use crate::prelude::*;

pub const ZENITH_PREFIX: &str = "[AutoDisconnect] ";

/// Automatically leave the server when in danger
pub struct AutoLeavePlugin;

impl Plugin for AutoLeavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SwarmState::default())
            .add_systems(PreUpdate, Self::handle_ping_packets)
            .add_systems(
                Update,
                (
                    Self::handle_add_entity_packets,
                    Self::handle_disconnect_events,
                    Self::handle_transfer_packets,
                    Self::handle_requeue,
                )
                    .chain()
                    .before(disconnect_listener)
                    .before(start_rejoin_on_disconnect),
            );
    }
}

#[derive(Component)]
pub struct GrimDisconnect;

impl AutoLeavePlugin {
    #[allow(clippy::cognitive_complexity)]
    pub fn handle_disconnect_events(
        mut events: EventReader<DisconnectEvent>,
        mut query: Query<&GameProfileComponent>,
        mut commands: Commands,
        swarm_state: Res<SwarmState>,
    ) {
        for event in events.read() {
            let Ok(game_profile) = query.get_mut(event.entity) else {
                continue;
            };

            let Some(reason) = &event.reason else {
                continue;
            };

            let username = &game_profile.name;
            info!("[{username}] Disconnect Reason: {reason}");

            let auto_reconnect = if str!(reason).starts_with(ZENITH_PREFIX) {
                info!("[{username}] AutoReconnect Disabled: ZenithProxy AutoDisconnect");
                (true, 10)
            } else if str!(reason).starts_with(LEAVE_PREFIX) {
                info!("[{username}] AutoReconnect Disabled: Leave Command");
                (false, 10)
            } else if str!(reason).starts_with("Connection throttled") {
                info!("[{username}] AutoReconnected too fast, waiting 30s...");
                (true, 30)
            } else {
                info!("[{username}] AutoReconnecting in 10s...");
                (true, 10)
            };

            commands.entity(event.entity).remove::<GrimDisconnect>();
            swarm_state
                .auto_reconnect
                .write()
                .insert(username.to_lowercase(), auto_reconnect);
        }
    }

    pub fn handle_add_entity_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        mut disconnect_events: EventWriter<DisconnectEvent>,
        query: Query<(&TabList, &GameProfileComponent, &LocalSettings)>,
        global_settings: Res<GlobalSettings>,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
                continue;
            };

            if packet.entity_type != EntityKind::Player {
                continue;
            }

            let Ok((tab_list, game_profile, local_settings)) = query.get(event.entity) else {
                continue;
            };

            let Some((uuid, info)) = tab_list.iter().find(|(uuid, _)| uuid == &&packet.uuid) else {
                continue;
            };

            let username = &game_profile.name;
            if global_settings.whitelist_only
                && local_settings.auto_leave.unknown_player
                && !global_settings.users.contains_key(uuid)
            {
                let name = &info.profile.name;
                let reason = format!("Unknown player in visual range: {name}");
                info!("[{username}] {reason}");
                info!("[{username}] Disabling AutoReconnect");

                disconnect_events.write(DisconnectEvent {
                    entity: event.entity,
                    reason: Some(FormattedText::from(reason)),
                });
            }
        }
    }

    pub fn handle_transfer_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        mut disconnect_events: EventWriter<DisconnectEvent>,
        query: Query<&GameProfileComponent>,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::Transfer(_) = event.packet.as_ref() else {
                continue;
            };

            let Ok(game_profile) = query.get(event.entity) else {
                continue;
            };

            let username = &game_profile.name;
            let reason = "Received transfer packet, disconnecting...";
            info!("[{username}] {reason}");

            disconnect_events.write(DisconnectEvent {
                entity: event.entity,
                reason: Some(FormattedText::from(reason)),
            });
        }
    }

    pub fn handle_ping_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        query: Query<&GrimDisconnect>,
        mut commands: Commands,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::Ping(packet) = event.packet.as_ref() else {
                continue;
            };

            if query.get(event.entity).is_ok() {
                continue;
            }

            commands.trigger(SendPacketEvent {
                sent_by: event.entity,
                packet:  ServerboundGamePacket::Pong(ServerboundPong { id: packet.id }),
            });
        }
    }

    pub fn handle_requeue(
        query: Query<(Entity, &GameTicks, &LocalSettings, &TabList), With<RawConnection>>,
        mut disconnect_events: EventWriter<DisconnectEvent>,
        mut commands: Commands,
    ) {
        query
            .iter()
            .filter(|(_, _, _, tab_list)| tab_list.len() > 1)
            .sorted_by_key(|(_, ticks, _, _)| ticks.0)
            .chunk_by(|(_, _, settings, _)| &settings.auto_pearl.location)
            .into_iter()
            .for_each(|(_, group)| {
                for (i, (entity, _, settings, _)) in group.enumerate() {
                    if i == 0 {
                        continue;
                    }

                    if !settings.auto_leave.auto_requeue {
                        continue;
                    }

                    if settings.auto_leave.grim_disconnect {
                        commands.entity(entity).insert(GrimDisconnect);
                    } else {
                        disconnect_events.write(DisconnectEvent {
                            entity,
                            reason: Some(FormattedText::from("Re-queueing...")),
                        });
                    }
                }
            });
    }
}
