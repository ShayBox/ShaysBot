use azalea::{
    app::{App, Plugin, Update},
    disconnect::DisconnectEvent,
    ecs::prelude::*,
    events::disconnect_listener,
    packet_handling::game::PacketEvent,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    FormattedText,
    GameProfileComponent,
    TabList,
};

use crate::prelude::*;

pub const ZENITH_PREFIX: &str = "[AutoDisconnect] ";

/// Automatically exit the process conditions are met.
pub struct AutoExitPlugin;

impl Plugin for AutoExitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SwarmState::default()).add_systems(
            Update,
            (
                Self::handle_add_entity_packets.before(disconnect_listener),
                Self::handle_disconnect_events,
            )
                .chain(),
        );
    }
}

impl AutoExitPlugin {
    pub fn handle_disconnect_events(
        mut events: EventReader<DisconnectEvent>,
        mut query: Query<(&GameProfileComponent, &LocalSettings)>,
        swarm_state: Res<SwarmState>,
    ) {
        for event in events.read() {
            let Ok((game_profile, local_settings)) = query.get_mut(event.entity) else {
                continue;
            };

            let Some(reason) = &event.reason else {
                continue;
            };

            let name = &game_profile.name;
            if local_settings.auto_leave.zenith_proxy && str!(reason).starts_with(ZENITH_PREFIX) {
                info!("[{name}] ZenithProxy Reason: {reason}");
                info!("[{name}] Disabling AutoReconnect");
                swarm_state
                    .auto_reconnect
                    .write()
                    .insert(game_profile.uuid, false);
            } else {
                info!("[{name}] Disconnect Reason: {reason}");
                info!("[{name}] AutoReconnect in 5s");
            }
        }
    }

    fn handle_add_entity_packets(
        mut packet_events: EventReader<PacketEvent>,
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

            if global_settings.whitelist
                && local_settings.auto_leave.unknown_player
                && !global_settings.whitelisted.contains_key(uuid)
            {
                let name = &info.profile.name;
                let username = &game_profile.name;
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
}
