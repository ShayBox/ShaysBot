use std::collections::HashMap;

use azalea::{
    app::{App, Plugin, PostUpdate, Update},
    auth::game_profile::GameProfile,
    ecs::prelude::*,
    packet_handling::game::PacketEvent,
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    world::MinecraftEntityId,
    TabList,
};

/// Tracks player profiles for visual range
pub struct PlayerProfilePlugin;

impl Plugin for PlayerProfilePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerProfiles::default())
            .add_systems(Update, Self::handle_add_entity_packets)
            .add_systems(PostUpdate, Self::handle_remove_entities_packets);
    }
}

#[derive(Clone, Component, Default, Resource)]
pub struct PlayerProfiles(pub HashMap<MinecraftEntityId, GameProfile>);

impl PlayerProfilePlugin {
    pub fn handle_add_entity_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut player_profiles: ResMut<PlayerProfiles>,
        mut query_profiles: Query<&mut PlayerProfiles>,
        mut commands: Commands,
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

            /* Insert to the global player profiles resource */
            player_profiles.0.insert(packet.id, info.profile.clone());

            /* Insert to or insert the local player profiles component */
            if let Ok(mut player_profiles) = query_profiles.get_mut(event.entity) {
                player_profiles.0.insert(packet.id, info.profile.clone());
            } else {
                let mut player_profiles = PlayerProfiles::default();
                player_profiles.0.insert(packet.id, info.profile.clone());
                commands.entity(event.entity).insert(player_profiles);
            }
        }
    }

    pub fn handle_remove_entities_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut player_profiles: ResMut<PlayerProfiles>,
        mut query: Query<&mut PlayerProfiles>,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
                continue;
            };

            for entity_id in &packet.entity_ids {
                /* Remove from the global player profiles resource */
                player_profiles.0.remove(entity_id);

                /* Remove from the local player profiles component */
                if let Ok(mut player_profiles) = query.get_mut(event.entity) {
                    player_profiles.0.remove(entity_id);
                }
            }
        }
    }
}
