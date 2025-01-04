use std::collections::HashMap;

use azalea::{
    app::{App, Plugin, PostUpdate, Update},
    blocks::BlockState,
    ecs::prelude::*,
    packet_handling::game::PacketEvent,
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    BlockPos,
};

/// Tracks block states.
pub struct BlockStatePlugin;

impl Plugin for BlockStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockStates::default())
            .add_systems(Update, Self::handle_block_update_packets)
            .add_systems(PostUpdate, Self::handle_block_break_packets);
    }
}

#[derive(Clone, Component, Default, Resource)]
pub struct BlockStates(pub HashMap<BlockPos, BlockState>);

impl BlockStatePlugin {
    pub fn handle_block_update_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut block_states: ResMut<BlockStates>,
        mut query: Query<&mut BlockStates>,
        mut commands: Commands,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
                continue;
            };

            /* Insert to the global block states resource */
            block_states.0.insert(packet.pos, packet.block_state);

            /* Insert to or insert the local block states component */
            if let Ok(mut block_states) = query.get_mut(event.entity) {
                block_states.0.insert(packet.pos, packet.block_state);
            } else {
                let mut block_states = BlockStates::default();
                block_states.0.insert(packet.pos, packet.block_state);
                commands.entity(event.entity).insert(block_states);
            }
        }
    }

    pub fn handle_block_break_packets(
        mut packet_events: EventReader<PacketEvent>,
        mut block_states: ResMut<BlockStates>,
        mut query: Query<&mut BlockStates>,
    ) {
        for event in packet_events.read() {
            let ClientboundGamePacket::BlockDestruction(packet) = event.packet.as_ref() else {
                continue;
            };

            /* Remove from the global block states resource */
            block_states.0.remove(&packet.pos);

            /* Remove from the local block states component */
            if let Ok(mut block_states) = query.get_mut(event.entity) {
                block_states.0.remove(&packet.pos);
            }
        }
    }
}
