use anyhow::{bail, Result};
use azalea::{
    app::{App, Plugin, Update},
    blocks::{properties::Open, Block},
    ecs::prelude::*,
    entity::Position,
    packet_handling::game::PacketEvent,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    BlockPos,
    GameProfileComponent,
    InstanceHolder,
    Vec3,
};
use derive_config::DeriveYamlConfig;
use uuid::Uuid;

use crate::{SettingsLock, Trapdoor, TrapdoorsLock};

pub struct PearlTrackerPlugin;

impl Plugin for PearlTrackerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_add_entity_packet,
                handle_block_update_packet,
                handle_remove_entities_packet,
            ),
        );
    }
}

pub fn handle_add_entity_packet(
    mut events: EventReader<PacketEvent>,
    mut query: Query<(&InstanceHolder, &GameProfileComponent, &TrapdoorsLock)>,
) {
    for event in events.read() {
        let Ok((holder, profile, trapdoors)) = query.get_mut(event.entity) else {
            continue;
        };

        let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
            continue;
        };

        if packet.entity_type != EntityKind::EnderPearl {
            return;
        }

        let Ok(block_pos) = find_nearest_trapdoor(packet.position, holder) else {
            return;
        };

        /* Player is outside visual range */
        let owner_uuid = if packet.data == 0 {
            Uuid::max()
        } else {
            info!("{}'s pearl at {block_pos}", profile.name);
            profile.uuid
        };

        let mut trapdoors = trapdoors.0.write();
        let new_trapdoor = Trapdoor::new(block_pos, packet.id, owner_uuid);

        if let Some(old_trapdoor) = trapdoors.0.get_mut(&packet.uuid) {
            if owner_uuid != Uuid::max() {
                *old_trapdoor = new_trapdoor;
            }
        } else {
            trapdoors.0.insert(packet.uuid, new_trapdoor);
        }

        trapdoors.save().expect("Failed to save trapdoors");
        drop(trapdoors);
    }
}
pub fn handle_block_update_packet(
    mut events: EventReader<PacketEvent>,
    mut query: Query<&TrapdoorsLock>,
) {
    for event in events.read() {
        let Ok(trapdoors) = query.get_mut(event.entity) else {
            continue;
        };

        if let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() {
            if let Some(open) = packet.block_state.property::<Open>() {
                if open {
                    return;
                }
            };

            let mut trapdoors = trapdoors.0.write();

            trapdoors
                .0
                .clone()
                .iter()
                .filter(|(_, trapdoor)| trapdoor.block_pos == packet.pos)
                .for_each(|(id, _)| {
                    trapdoors.0.remove(id);
                });

            trapdoors.save().expect("Failed to save trapdoors");
            drop(trapdoors);
        }
    }
}

pub fn handle_remove_entities_packet(
    mut events: EventReader<PacketEvent>,
    mut query: Query<(&TrapdoorsLock, &Position, &SettingsLock)>,
) {
    for event in events.read() {
        let Ok((trapdoors, position, settings)) = query.get_mut(event.entity) else {
            continue;
        };

        if let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() {
            let view_distance = settings.0.read().pearl_view_distance;
            let view_distance_sqr = f64::from(view_distance.pow(2));
            let mut trapdoors = trapdoors.0.write();

            trapdoors.0.retain(|_, trapdoor| {
                let trapdoor_pos = trapdoor.block_pos.to_vec3_floored();
                let distance_sqr = trapdoor_pos.distance_to_sqr(position);

                !(packet.entity_ids.contains(&trapdoor.entity_id)
                    && distance_sqr <= view_distance_sqr)
            });

            trapdoors.save().expect("Failed to save trapdoors");
            drop(trapdoors);
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn find_nearest_trapdoor(position: Vec3, holder: &InstanceHolder) -> Result<BlockPos> {
    let instance = holder.instance.read();

    let x = position.x.trunc() as i32;
    let z = position.z.trunc() as i32;
    let min_y = (position.y - 5.0).trunc() as i32;
    let max_y = (position.y + 5.0).ceil() as i32;
    for y in min_y..max_y {
        let pos = BlockPos::new(x, y, z);
        let Some(state) = instance.get_block_state(&pos) else {
            continue;
        };

        if Box::<dyn Block>::from(state).id().ends_with("_trapdoor") {
            return Ok(pos);
        }
    }

    drop(instance);
    bail!("Unable to a find nearby trapdoor")
}
