use std::sync::Arc;

use azalea::{
    app::{App, Plugin, Update},
    chunks::handle_receive_chunk_events,
    core::direction::Direction,
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    interact::handle_swing_arm_event,
    inventory::InventorySet,
    mining::MiningSet,
    packet_handling::game::{handle_send_packet_event, SendPacketEvent},
    pathfinder::{
        goals::ReachBlockPosGoal,
        goto_listener,
        moves::default_move,
        GotoEvent,
        Pathfinder,
    },
    physics::PhysicsSet,
    prelude::*,
    protocol::packets::game::{
        serverbound_interact_packet::InteractionHand,
        serverbound_use_item_on_packet::{BlockHit, ServerboundUseItemOnPacket},
        ServerboundGamePacket,
    },
    BlockPos,
    InstanceHolder,
    Vec3,
};

pub struct AutoPearlPlugin;

impl Plugin for AutoPearlPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PearlEvent>().add_systems(
            Update,
            (
                handle_pearl_event,
                handle_pearl_goto
                    .before(goto_listener)
                    .after(handle_receive_chunk_events),
                handle_pearl_pull
                    .before(handle_send_packet_event)
                    .before(handle_swing_arm_event)
                    .after(InventorySet)
                    .after(PhysicsSet)
                    .after(MiningSet),
            ),
        );
    }
}

#[derive(Event)]
pub struct PearlEvent {
    pub entity:    Entity,
    pub block_pos: BlockPos,
}

pub fn handle_pearl_event(mut events: EventReader<PearlEvent>, mut commands: Commands) {
    for event in events.read() {
        commands.entity(event.entity).insert(PearlGoto {
            block_pos: event.block_pos,
        });
    }
}

#[derive(Component)]
pub struct PearlGoto {
    pub block_pos: BlockPos,
}

type GotoQueryData<'a> = (Entity, &'a PearlGoto, &'a Pathfinder, &'a InstanceHolder);
type GotoQueryFilter = (With<Player>, With<LocalEntity>, With<PearlGoto>);

pub fn handle_pearl_goto(
    mut query: Query<GotoQueryData, GotoQueryFilter>,
    mut goto_events: EventWriter<GotoEvent>,
    mut commands: Commands,
) {
    for (entity, pearl, pathfinder, holder) in &mut query {
        if let Some(_goal) = &pathfinder.goal {
            continue;
        }

        let goal = ReachBlockPosGoal {
            chunk_storage: holder.instance.read().chunks.clone(),
            pos:           pearl.block_pos,
        };

        goto_events.send(GotoEvent {
            entity,
            goal: Arc::new(goal),
            successors_fn: default_move,
            allow_mining: false,
        });

        commands.entity(entity).remove::<PearlGoto>();
        commands.entity(entity).insert(PearlPull {
            block_pos: pearl.block_pos,
        });
    }
}

#[derive(Component)]
pub struct PearlPull {
    pub block_pos: BlockPos,
}

type PullQueryData<'a> = (Entity, &'a PearlPull, &'a Pathfinder);
type PullQueryFilter = (With<Player>, With<LocalEntity>, With<PearlPull>);

pub fn handle_pearl_pull(
    mut query: Query<PullQueryData, PullQueryFilter>,
    mut packet_events: EventWriter<SendPacketEvent>,
    mut commands: Commands,
) {
    for (entity, pearl, pathfinder) in &mut query {
        if let Some(_goal) = &pathfinder.goal {
            continue;
        }

        let packet = ServerboundGamePacket::UseItemOn(ServerboundUseItemOnPacket {
            hand:      InteractionHand::MainHand,
            block_hit: BlockHit {
                block_pos: pearl.block_pos,
                direction: Direction::Down,
                location:  Vec3 {
                    x: f64::from(pearl.block_pos.x) + 0.5,
                    y: f64::from(pearl.block_pos.y) + 0.5,
                    z: f64::from(pearl.block_pos.z) + 0.5,
                },
                inside:    false,
            },
            sequence:  0,
        });

        packet_events.send(SendPacketEvent { entity, packet });
        commands.entity(entity).remove::<PearlPull>();
    }
}
