use azalea::{
    app::{App, Plugin, PostUpdate},
    blocks::{properties::Open, Block},
    ecs::prelude::*,
    entity::{metadata::Player, Position},
    events::packet_listener,
    packet::game::ReceiveGamePacketEvent,
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

/// Tracks ender pearls for new chambers
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
pub struct ResendPacketEvent(ReceiveGamePacketEvent);

impl EnderPearlPlugin {
    pub fn handle_resend_packets(
        mut resend_packet_events: EventReader<ResendPacketEvent>,
        mut packet_events: EventWriter<ReceiveGamePacketEvent>,
    ) {
        for event in resend_packet_events.read().cloned() {
            packet_events.write(event.0);
        }
    }

    /// # Panics
    /// Will panic if `MinecraftEntityId` out of bounds.
    /// Will panic of `Settings::save` fails.
    #[allow(clippy::cognitive_complexity)]
    pub fn handle_add_entity_packet(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
        mut query: Query<(&InstanceHolder, &LocalSettings)>,
        mut pearl_goto_events: EventWriter<PearlGotoEvent>,
        mut resend_packet_events: EventWriter<ResendPacketEvent>,
        mut stasis_chambers: ResMut<StasisChambers>,
        mut msg_events: EventWriter<MsgEvent>,
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

            let Some(block_pos) = find_block_pos(packet.position, holder, "_trapdoor") else {
                continue;
            };

            let owner_uuid = if packet.data == 0 {
                info!("Unknown player's pearl at {block_pos}");
                Uuid::max() /* Owner offline */
            } else if let Some((_, profile)) =
                player_profiles.iter().find(|(id, _)| id.0 == packet.data)
            {
                info!("{}'s pearl at {block_pos}", profile.name);
                profile.uuid /* Owner in visual range */
            } else {
                resend_packet_events.write(ResendPacketEvent(event));
                continue; /* Owner not in visual range */
            };

            debug!("Fishdar: {packet:#?}");
            let new_chamber = StasisChamber {
                block_pos,
                entity_id: packet.id,
                owner_uuid,
                location: local_settings.auto_pearl.location.clone(),
            };

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
                msg_events.write(MsgEvent {
                    entity: Some(event.entity),
                    sender: CmdSender::Minecraft(owner_uuid),
                    source: CmdSource::Minecraft(None),
                    status: 402,
                    content: format!(
                        "Your free trial has expired, please purchase WinRAR license: Max {limit} pearls"
                    ),
                });
                pearl_goto_events.write(PearlGotoEvent(PearlEvent {
                    entity: event.entity,
                    idle_goal: local_settings.auto_pearl.idle_goal.clone(),
                    block_pos,
                    owner_uuid,
                }));
            }
        }
    }

    /// # Panics
    /// Will panic of `Settings::save` fails.
    pub fn handle_block_update_packets(
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
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
            }

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
        mut packet_events: EventReader<ReceiveGamePacketEvent>,
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
pub fn find_block_pos(position: Vec3, holder: &InstanceHolder, pat: &str) -> Option<BlockPos> {
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

        if Box::<dyn Block>::from(state).id().ends_with(pat) {
            return Some(pos);
        }
    }

    drop(instance);
    None
}
