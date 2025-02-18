use azalea::{
    app::{App, Plugin, Update},
    disconnect::DisconnectEvent,
    ecs::prelude::*,
    events::disconnect_listener,
    packet_handling::game::{PacketEvent, SendPacketEvent},
    protocol::packets::game::{ClientboundGamePacket, ServerboundGamePacket, ServerboundPong},
    registry::EntityKind,
    FormattedText,
    GameProfileComponent,
    TabList,
};

use crate::prelude::*;

pub const ZENITH_PREFIX: &str = "[AutoDisconnect] ";

/// Automatically leave the server when in danger
pub struct AutoLeavePlugin;

impl Plugin for AutoLeavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SwarmState::default()).add_systems(
            Update,
            (
                Self::handle_add_entity_packets.before(disconnect_listener),
                Self::handle_disconnect_events,
                Self::handle_ping_packets,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct ForceDisconnect;

#[derive(Component)]
pub struct SkipDisconnect;

impl AutoLeavePlugin {
    #[allow(clippy::cognitive_complexity)]
    pub fn handle_disconnect_events(
        mut events: EventReader<DisconnectEvent>,
        mut query: Query<(&GameProfileComponent, &LocalSettings)>,
        mut commands: Commands,
        swarm_state: Res<SwarmState>,
    ) {
        for event in events.read() {
            let Ok((game_profile, local_settings)) = query.get_mut(event.entity) else {
                continue;
            };

            let Some(reason) = &event.reason else {
                continue;
            };

            let username = &game_profile.name;
            info!("[{username}] Disconnect Reason: {reason}");

            let auto_reconnect = if local_settings.auto_leave.zenith_proxy
                && str!(reason).starts_with(ZENITH_PREFIX)
            {
                info!("[{username}] AutoReconnect Disabled: Zenith Proxy");
                (false, 5)
            } else if str!(reason).starts_with(LEAVE_PREFIX) {
                info!("[{username}] AutoReconnect Disabled: Leave Command");
                (false, 5)
            } else if str!(reason).starts_with("Connection throttled") {
                info!("[{username}] AutoReconnected too fast, waiting 30s...");
                (true, 30)
            } else {
                info!("[{username}] AutoReconnecting in 5s...");
                (true, 5)
            };

            commands.entity(event.entity).remove::<ForceDisconnect>();
            swarm_state
                .auto_reconnect
                .write()
                .insert(username.to_lowercase(), auto_reconnect);
        }
    }

    pub fn handle_add_entity_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut disconnect_events: EventWriter<DisconnectEvent>,
        mut commands: Commands,
        local_players: Query<(Entity, &GameProfileComponent, &LocalSettings)>,
        query: Query<(&TabList, &GameProfileComponent, &LocalSettings)>,
        skip_disconnect: Query<&SkipDisconnect>,
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

            if local_settings.auto_leave.auto_requeue {
                let local_location = &local_settings.auto_pearl.location;
                if let Some((entity, _, _)) = local_players.iter().find(|(_, profile, settings)| {
                    let location = &settings.auto_pearl.location;
                    &profile.uuid == uuid && location == local_location
                }) {
                    if skip_disconnect.contains(event.entity) {
                        commands.entity(event.entity).remove::<SkipDisconnect>();
                    } else {
                        commands.entity(entity).insert(SkipDisconnect);
                        if local_settings.auto_leave.zenith_proxy {
                            info!("[{username}] Forcefully re-queueing...");
                            commands.entity(event.entity).insert(ForceDisconnect);
                        } else {
                            disconnect_events.send(DisconnectEvent {
                                entity: event.entity,
                                reason: Some(FormattedText::from("Automatically re-queueing")),
                            });
                        }
                    }
                }
            }

            if global_settings.whitelist_only
                && local_settings.auto_leave.unknown_player
                && !global_settings.users.contains_key(uuid)
            {
                let name = &info.profile.name;
                let reason = format!("Unknown player in visual range: {name}");
                info!("[{username}] {reason}");
                info!("[{username}] Disabling AutoReconnect");

                disconnect_events.send(DisconnectEvent {
                    entity: event.entity,
                    reason: Some(FormattedText::from(reason)),
                });
            }
        }
    }

    pub fn handle_ping_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut send_packet_events: EventWriter<SendPacketEvent>,
        query: Query<&ForceDisconnect>,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::Ping(packet) = event.packet.as_ref() else {
                continue;
            };

            if query.get(event.entity).is_ok() {
                continue;
            }

            send_packet_events.send(SendPacketEvent {
                sent_by: event.entity,
                packet:  ServerboundGamePacket::Pong(ServerboundPong { id: packet.id }),
            });
        }
    }
}
