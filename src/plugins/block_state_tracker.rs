use std::collections::HashMap;

use azalea::{
    app::{App, Plugin, PostUpdate, PreUpdate},
    blocks::BlockState,
    ecs::prelude::*,
    packet_handling::game::PacketEvent,
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    BlockPos,
};

pub struct BlockStateTrackerPlugin;

impl Plugin for BlockStateTrackerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockStates::default())
            .add_systems(PreUpdate, handle_block_update_packet)
            .add_systems(PostUpdate, handle_block_break_packet);
    }
}

#[derive(Default, Resource)]
pub struct BlockStates(pub HashMap<BlockPos, BlockState>);

fn handle_block_update_packet(
    mut packet_events: EventReader<PacketEvent>,
    mut block_states: ResMut<BlockStates>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
            continue;
        };

        block_states.0.insert(packet.pos, packet.block_state);
    }
}

fn handle_block_break_packet(
    mut packet_events: EventReader<PacketEvent>,
    mut block_states: ResMut<BlockStates>,
) {
    for event in packet_events.read() {
        let ClientboundGamePacket::BlockDestruction(packet) = event.packet.as_ref() else {
            continue;
        };

        block_states.0.remove(&packet.pos);
    }
}
