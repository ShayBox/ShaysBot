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

use crate::{Settings, SwarmState};

/// Automatically exit the process conditions are met.
pub struct AutoExitPlugin;

impl Plugin for AutoExitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SwarmState::default()).add_systems(
            Update,
            (
                handle_add_entity_packet.before(disconnect_listener),
                handle_disconnect_event,
            )
                .chain(),
        );
    }
}

const ZENITH_REASON: &str = "AutoDisconnect";

pub fn handle_disconnect_event(
    mut events: EventReader<DisconnectEvent>,
    mut query: Query<&GameProfileComponent>,
    swarm_state: Res<SwarmState>,
) {
    for event in events.read() {
        let Ok(profile) = query.get_mut(event.entity) else {
            continue;
        };

        let Some(reason) = &event.reason else {
            continue;
        };

        if reason.to_string().starts_with(ZENITH_REASON) {
            info!("[AutoReconnect] Disabled for {}", profile.name);
            swarm_state
                .auto_reconnect
                .write()
                .insert(profile.uuid, false);
        } else {
            info!("[AutoReconnect] Disconnect Reason: {}", reason.to_ansi());
        }
    }
}

fn handle_add_entity_packet(
    mut packet_events: EventReader<PacketEvent>,
    mut disconnect_events: EventWriter<DisconnectEvent>,
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

        let Some((uuid, info)) = tab_list.iter().find(|(uuid, _)| uuid == &&packet.uuid) else {
            continue;
        };

        if !settings.whitelisted.is_empty()
            && !settings.whitelisted.contains_key(uuid)
            && settings.unknown_player_auto_exit
        {
            let name = &info.profile.name;
            let reason = format!("{ZENITH_REASON} - Unknown player in visual range: {name}");
            disconnect_events.send(DisconnectEvent {
                entity: event.entity,
                reason: Some(FormattedText::from(reason)),
            });
        }
    }
}
