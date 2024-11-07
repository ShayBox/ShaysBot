use azalea::{
    app::{App, Plugin, PostUpdate},
    blocks::{properties::Open, Block},
    ecs::prelude::*,
    entity::{metadata::Player, Position},
    packet_handling::game::PacketEvent,
    packet_listener,
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    world::MinecraftEntityId,
    BlockPos,
    GameProfileComponent,
    InstanceHolder,
    Vec3,
};
use derive_config::DeriveYamlConfig;
use uuid::Uuid;

use crate::{Settings, Trapdoor, Trapdoors};

/// Keep track of thrown pearls inside of stasis chambers for `AutoPearlPlugin`
pub struct PearlTrackerPlugin;

impl Plugin for PearlTrackerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PacketEvents::default()).add_systems(
            PostUpdate,
            (
                handle_packet_events.before(packet_listener),
                handle_add_entity_packet,
                handle_block_update_packet,
                handle_remove_entities_packet,
            ),
        );
    }
}

#[derive(Default, Resource)]
pub struct PacketEvents(pub Vec<PacketEvent>);

pub fn handle_packet_events(
    mut events: ResMut<PacketEvents>,
    mut packet_events: EventWriter<PacketEvent>,
) {
    for event in events.0.drain(..) {
        packet_events.send(event);
    }
}

/// # Panics
/// Will panic if `MinecraftEntityId` is out of bounds.
/// Will panic of `Settings::save` fails.
pub fn handle_add_entity_packet(
    mut events: EventReader<PacketEvent>,
    mut packet_events: ResMut<PacketEvents>,
    mut query: Query<&InstanceHolder>,
    mut trapdoors: ResMut<Trapdoors>,
    profiles: Query<(&MinecraftEntityId, &GameProfileComponent), With<Player>>,
) {
    for event in events.read() {
        let Ok(holder) = query.get_mut(event.entity) else {
            continue;
        };

        let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
            continue;
        };

        if packet.entity_type != EntityKind::EnderPearl {
            continue;
        }

        let Some(block_pos) = find_nearest_trapdoor(packet.position, holder) else {
            continue;
        };

        /* Player is outside visual range */
        let owner_uuid = if packet.data == 0 {
            Uuid::max()
        } else if let Some((_, profile)) = profiles
            .iter()
            .find(|(id, _)| i32::try_from(id.0).unwrap() == packet.data)
        {
            info!("{}'s pearl at {block_pos}", profile.name);
            profile.uuid
        } else {
            // The owner's uuid was sent, but the owner wasn't found in the entity list
            // Send the event back and try again next update until the owner is received
            packet_events.0.push(event.clone());
            continue;
        };

        let new_trapdoor = Trapdoor::new(block_pos, packet.id, owner_uuid);

        if let Some(old_trapdoor) = trapdoors.0.get_mut(&packet.uuid) {
            if owner_uuid != Uuid::max() {
                *old_trapdoor = new_trapdoor;
            }
        } else {
            trapdoors.0.insert(packet.uuid, new_trapdoor);
        }

        trapdoors.save().expect("Failed to save trapdoors");
    }
}

/// # Panics
/// Will panic of `Settings::save` fails.
pub fn handle_block_update_packet(
    mut events: EventReader<PacketEvent>,
    mut trapdoors: ResMut<Trapdoors>,
) {
    for event in events.read() {
        let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
            continue;
        };

        if let Some(open) = packet.block_state.property::<Open>() {
            if open {
                return;
            }
        };

        trapdoors
            .0
            .clone()
            .iter()
            .filter(|(_, trapdoor)| trapdoor.block_pos == packet.pos)
            .for_each(|(id, _)| {
                trapdoors.0.remove(id);
            });

        trapdoors.save().expect("Failed to save trapdoors");
    }
}

/// # Panics
/// Will panic of `Settings::save` fails.
pub fn handle_remove_entities_packet(
    mut events: EventReader<PacketEvent>,
    mut query: Query<&Position>,
    mut trapdoors: ResMut<Trapdoors>,
    settings: Res<Settings>,
) {
    for event in events.read() {
        let Ok(position) = query.get_mut(event.entity) else {
            continue;
        };

        let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
            continue;
        };

        let view_distance = settings.pearl_view_distance;
        let view_distance_sqr = f64::from(view_distance.pow(2));

        trapdoors.0.retain(|_, trapdoor| {
            let trapdoor_pos = trapdoor.block_pos.to_vec3_floored();
            let distance_sqr = trapdoor_pos.distance_to_sqr(position);

            !(packet.entity_ids.contains(&trapdoor.entity_id) && distance_sqr <= view_distance_sqr)
        });

        trapdoors.save().expect("Failed to save trapdoors");
    }
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn find_nearest_trapdoor(position: Vec3, holder: &InstanceHolder) -> Option<BlockPos> {
    let instance = holder.instance.read();

    let x = position.x.floor() as i32;
    let z = position.z.floor() as i32;
    let min_y = position.y.floor() as i32 - 5;
    let max_y = position.y.ceil() as i32 + 5;
    for y in min_y..max_y {
        let pos = BlockPos::new(x, y, z);
        let Some(state) = instance.get_block_state(&pos) else {
            continue;
        };

        if Box::<dyn Block>::from(state).id().ends_with("_trapdoor") {
            return Some(pos);
        }
    }

    drop(instance);
    None
}
