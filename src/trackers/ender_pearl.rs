use azalea::{
    app::{App, Plugin, PostUpdate},
    blocks::{properties::Open, Block},
    ecs::prelude::*,
    entity::{metadata::Player, Position},
    events::packet_listener,
    packet_handling::game::PacketEvent,
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    registry::EntityKind,
    world::MinecraftEntityId,
    BlockPos,
    GameProfileComponent,
    InstanceHolder,
    Vec3,
};
use uuid::Uuid;

use crate::prelude::*;

/// Tracks ender pearls.
pub struct EnderPearlPlugin;

impl Plugin for EnderPearlPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResendPacketEvent>().add_systems(
            PostUpdate,
            (
                Self::handle_add_entity_packet,
                Self::handle_block_update_packets,
                Self::handle_remove_entities_packets,
                Self::handle_resend_packets.before(packet_listener),
            ),
        );
    }
}

#[derive(Clone, Event)]
pub struct ResendPacketEvent(PacketEvent);

impl EnderPearlPlugin {
    pub fn handle_resend_packets(
        mut resend_packet_events: EventReader<ResendPacketEvent>,
        mut packet_events: EventWriter<PacketEvent>,
    ) {
        for event in resend_packet_events.read().cloned() {
            packet_events.send(event.0);
        }
    }

    /// # Panics
    /// Will panic if `MinecraftEntityId` is out of bounds.
    /// Will panic of `Settings::save` fails.
    pub fn handle_add_entity_packet(
        mut packet_events: EventReader<PacketEvent>,
        mut query: Query<(&InstanceHolder, &LocalSettings)>,
        mut pearl_goto_events: EventWriter<PearlGotoEvent>,
        mut resend_packet_events: EventWriter<ResendPacketEvent>,
        mut stasis_chambers: ResMut<StasisChambers>,
        mut whisper_events: EventWriter<WhisperEvent>,
        player_profiles: Query<(&MinecraftEntityId, &GameProfileComponent), With<Player>>,
    ) {
        for event in packet_events.read().cloned() {
            let Ok((holder, local_settings)) = query.get_mut(event.entity) else {
                continue;
            };

            let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
                continue;
            };

            if packet.entity_type != EntityKind::EnderPearl {
                continue;
            }

            trace!("Fishdar: {packet:#?}");
            let Some(block_pos) = find_trapdoor_pos(packet.position, holder) else {
                continue;
            };

            let owner_uuid = if packet.data == 0 {
                info!("Unknown player's pearl at {block_pos}");
                Uuid::max() /* Owner is offline */
            } else if let Some((_, profile)) =
                player_profiles.iter().find(|(id, _)| id.0 == packet.data)
            {
                info!("{}'s pearl at {block_pos}", profile.name);
                profile.uuid /* Owner is in visual range */
            } else {
                resend_packet_events.send(ResendPacketEvent(event));
                continue; /* Owner is not in visual range */
            };

            let new_chamber = StasisChamber::new(
                block_pos,
                packet.id,
                owner_uuid,
                local_settings.auto_pearl.location.clone(),
            );

            stasis_chambers
                .0
                .entry(packet.uuid)
                .and_modify(|old_chamber| {
                    if owner_uuid != Uuid::max() {
                        *old_chamber = new_chamber.clone();
                    }
                })
                .or_insert(new_chamber);

            stasis_chambers
                .save()
                .expect("Failed to save stasis chambers");

            if owner_uuid == Uuid::max() {
                continue; /* Don't pull random unknown pearls */
            }

            let limit = local_settings.auto_pearl.pearl_limit;
            let count = stasis_chambers
                .0
                .values()
                .filter(|chamber| chamber.location == local_settings.auto_pearl.location)
                .filter(|chamber| chamber.owner_uuid == owner_uuid)
                .count();

            debug!("Count: {count} | Limit: {limit}");
            if count > limit {
                whisper_events.send(WhisperEvent {
                    entity: event.entity,
                    sender: CommandSender::Minecraft(owner_uuid),
                    source: CommandSource::Minecraft(None),
                    status: 402,
                    content: format!(
                        "Your free trial has expired, please purchase WinRAR license: Max {limit} pearls"
                    ),
                });
                pearl_goto_events.send(PearlGotoEvent(PearlEvent {
                    entity: event.entity,
                    idle_goal: local_settings.auto_pearl.idle_goal.clone(),
                    block_pos,
                    owner_uuid,
                }));
            };
        }
    }

    /// # Panics
    /// Will panic of `Settings::save` fails.
    pub fn handle_block_update_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut stasis_chambers: ResMut<StasisChambers>,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
                continue;
            };

            if let Some(open) = packet.block_state.property::<Open>() {
                if open {
                    return;
                }
            };

            stasis_chambers
                .0
                .clone()
                .iter()
                .filter(|(_, chamber)| chamber.block_pos == packet.pos)
                .for_each(|(id, _)| {
                    stasis_chambers.0.remove(id);
                });

            stasis_chambers
                .save()
                .expect("Failed to save stasis chambers");
        }
    }

    /// # Panics
    /// Will panic of `Settings::save` fails.
    pub fn handle_remove_entities_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut player_positions: Query<&Position>,
        mut stasis_chambers: ResMut<StasisChambers>,
        global_settings: Res<GlobalSettings>,
    ) {
        for event in packet_events.read() {
            let Ok(position) = player_positions.get_mut(event.entity) else {
                continue;
            };

            let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
                continue;
            };

            let view_distance = global_settings.pearl_view_distance;
            let view_distance_sqr = f64::from(view_distance.pow(2));

            stasis_chambers.0.retain(|_, chamber| {
                let chamber_pos = chamber.block_pos.to_vec3_floored();
                let distance_sqr = chamber_pos.distance_squared_to(position);

                !(packet.entity_ids.contains(&chamber.entity_id)
                    && distance_sqr <= view_distance_sqr)
            });

            stasis_chambers
                .save()
                .expect("Failed to save stasis chambers");
        }
    }
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn find_trapdoor_pos(position: Vec3, holder: &InstanceHolder) -> Option<BlockPos> {
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
