use std::sync::Arc;

use azalea::{
    app::{App, Plugin, Update},
    chunks::handle_receive_chunk_events,
    core::direction::Direction,
    ecs::prelude::*,
    interact::handle_swing_arm_event,
    inventory::InventorySet,
    mining::MiningSet,
    packet_handling::game::{handle_send_packet_event, SendPacketEvent},
    pathfinder::{
        goals::{RadiusGoal, ReachBlockPosGoal},
        goto_listener,
        moves::default_move,
        GotoEvent,
        Pathfinder,
    },
    physics::PhysicsSet,
    prelude::*,
    protocol::packets::game::{
        s_interact::InteractionHand,
        s_use_item_on::BlockHit,
        ServerboundGamePacket,
        ServerboundUseItemOn,
    },
    BlockPos,
    InstanceHolder,
    TabList,
    Vec3,
};
use uuid::Uuid;

use crate::settings::IdleGoal;

/// Automatically pull stasis chamber pearls
pub struct AutoPearlPlugin;

impl Plugin for AutoPearlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PendingPearlEvents::default())
            .add_event::<PearlGotoEvent>()
            .add_event::<PearlPullEvent>()
            .add_systems(
                Update,
                (
                    handle_pearl_goto_event
                        .before(goto_listener)
                        .after(handle_receive_chunk_events),
                    handle_pearl_pull_event
                        .before(handle_send_packet_event)
                        .before(handle_swing_arm_event)
                        .after(InventorySet)
                        .after(PhysicsSet)
                        .after(MiningSet),
                    process_pending_pearl_events,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Event)]
pub struct PearlGotoEvent {
    pub entity:     Entity,
    pub idle_goal:  IdleGoal,
    pub block_pos:  BlockPos,
    pub owner_uuid: Uuid,
}

pub fn handle_pearl_goto_event(
    mut goto_events: EventWriter<GotoEvent>,
    mut pearl_goto_events: EventReader<PearlGotoEvent>,
    mut pearl_pull_events: EventWriter<PearlPullEvent>,
    mut pearl_pending_events: ResMut<PendingPearlEvents>,
    mut query: Query<(&Pathfinder, &InstanceHolder)>,
) {
    for event in pearl_goto_events.read() {
        let Ok((pathfinder, holder)) = query.get_mut(event.entity) else {
            continue;
        };

        if let Some(_goal) = &pathfinder.goal {
            pearl_pending_events.goto.push(event.clone());
            continue;
        }

        let goal = ReachBlockPosGoal {
            chunk_storage: holder.instance.read().chunks.clone(),
            pos:           event.block_pos,
        };

        goto_events.send(GotoEvent {
            entity:        event.entity,
            goal:          Arc::new(goal),
            successors_fn: default_move,
            allow_mining:  false,
        });

        pearl_pull_events.send(event.into());
    }
}

#[derive(Clone, Event)]
pub struct PearlPullEvent {
    pub entity:     Entity,
    pub idle_goal:  IdleGoal,
    pub block_pos:  BlockPos,
    pub owner_uuid: Uuid,
}

pub fn handle_pearl_pull_event(
    mut goto_events: EventWriter<GotoEvent>,
    mut pearl_pending_events: ResMut<PendingPearlEvents>,
    mut pearl_pull_events: EventReader<PearlPullEvent>,
    mut send_packet_events: EventWriter<SendPacketEvent>,
    mut query: Query<(&Pathfinder, &TabList)>,
) {
    for event in pearl_pull_events.read() {
        let Ok((pathfinder, tab_list)) = query.get_mut(event.entity) else {
            continue;
        };

        if let Some(_goal) = &pathfinder.goal {
            pearl_pending_events.pull.push(event.clone());
            continue;
        }

        if !tab_list.contains_key(&event.owner_uuid) {
            pearl_pending_events.goto.push(event.into());
            continue;
        }

        let packet = ServerboundGamePacket::UseItemOn(ServerboundUseItemOn {
            hand:      InteractionHand::MainHand,
            block_hit: BlockHit {
                block_pos: event.block_pos,
                direction: Direction::Down,
                location:  Vec3 {
                    x: f64::from(event.block_pos.x) + 0.5,
                    y: f64::from(event.block_pos.y) + 0.5,
                    z: f64::from(event.block_pos.z) + 0.5,
                },
                inside:    false,
            },
            sequence:  0,
        });

        send_packet_events.send(SendPacketEvent {
            sent_by: event.entity,
            packet,
        });

        if event.idle_goal != IdleGoal::default() {
            goto_events.send(GotoEvent {
                entity:        event.entity,
                allow_mining:  false,
                successors_fn: default_move,
                goal:          Arc::new(RadiusGoal {
                    pos:    event.idle_goal.coords,
                    radius: event.idle_goal.radius + 1.0,
                }),
            });
        }
    }
}

#[derive(Default, Resource)]
pub struct PendingPearlEvents {
    goto: Vec<PearlGotoEvent>,
    pull: Vec<PearlPullEvent>,
}

pub fn process_pending_pearl_events(
    mut pearl_goto_events: EventWriter<PearlGotoEvent>,
    mut pearl_pull_events: EventWriter<PearlPullEvent>,
    mut pearl_pending_events: ResMut<PendingPearlEvents>,
    mut query: Query<&Pathfinder>,
) {
    for pathfinder in &mut query {
        if let Some(_goal) = &pathfinder.goal {
            continue;
        }

        if let Some(event) = pearl_pending_events.goto.pop() {
            pearl_goto_events.send(event);
        }

        if let Some(event) = pearl_pending_events.pull.pop() {
            pearl_pull_events.send(event);
        }
    }
}

impl From<&PearlGotoEvent> for PearlPullEvent {
    fn from(event: &PearlGotoEvent) -> Self {
        Self {
            entity:     event.entity,
            idle_goal:  event.idle_goal.clone(),
            block_pos:  event.block_pos,
            owner_uuid: event.owner_uuid,
        }
    }
}

impl From<&PearlPullEvent> for PearlGotoEvent {
    fn from(event: &PearlPullEvent) -> Self {
        Self {
            entity:     event.entity,
            idle_goal:  event.idle_goal.clone(),
            block_pos:  event.block_pos,
            owner_uuid: event.owner_uuid,
        }
    }
}
