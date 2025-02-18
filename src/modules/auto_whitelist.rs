use azalea::{
    app::{App, Plugin, Update},
    packet_handling::game::PacketEvent,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
};

use crate::prelude::*;

/// Automatically whitelist players that enter range
pub struct AutoWhitelistPlugin;

impl Plugin for AutoWhitelistPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::handle_add_entity_packets);
    }
}

impl AutoWhitelistPlugin {
    pub fn handle_add_entity_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut global_settings: ResMut<GlobalSettings>,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
                continue;
            };

            if packet.entity_type != EntityKind::Player {
                continue;
            }

            if global_settings.users.contains_key(&packet.uuid) {
                continue;
            }

            if global_settings.whitelist_in_range {
                debug!("Adding {} to whitelist", packet.uuid);
                global_settings.users.insert(packet.uuid, User::default());
            }
        }
    }
}
