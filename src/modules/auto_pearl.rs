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

use crate::prelude::*;

/// Automatically pull stasis chamber pearls
pub struct AutoPearlPlugin;

impl Plugin for AutoPearlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ResendPearlEvents::default())
            .add_event::<PearlGotoEvent>()
            .add_event::<PearlPullEvent>()
            .add_systems(
                Update,
                (
                    Self::handle_resend_pearl_events,
                    Self::handle_goto_pearl_events
                        .before(goto_listener)
                        .after(handle_receive_chunk_events),
                    Self::handle_pull_pearl_events
                        .before(handle_send_packet_event)
                        .before(handle_swing_arm_event)
                        .after(InventorySet)
                        .after(PhysicsSet)
                        .after(MiningSet),
                )
                    .chain(),
            );
    }
}

#[derive(Clone)]
pub struct PearlEvent {
    pub entity:     Entity,
    pub idle_goal:  IdleGoal,
    pub block_pos:  BlockPos,
    pub owner_uuid: Uuid,
}

#[derive(Clone, Deref, DerefMut, Event)]
pub struct PearlGotoEvent(pub PearlEvent);

#[derive(Clone, Deref, DerefMut, Event)]
pub struct PearlPullEvent(PearlEvent);

#[derive(Default, Resource)]
pub struct ResendPearlEvents {
    goto: Vec<PearlGotoEvent>,
    pull: Vec<PearlPullEvent>,
}

impl AutoPearlPlugin {
    pub fn handle_resend_pearl_events(
        mut pearl_goto_events: EventWriter<PearlGotoEvent>,
        mut pearl_pull_events: EventWriter<PearlPullEvent>,
        mut pearl_pending_events: ResMut<ResendPearlEvents>,
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

    pub fn handle_goto_pearl_events(
        mut goto_events: EventWriter<GotoEvent>,
        mut pearl_goto_events: EventReader<PearlGotoEvent>,
        mut pearl_pull_events: EventWriter<PearlPullEvent>,
        mut pearl_pending_events: ResMut<ResendPearlEvents>,
        mut query: Query<(&Pathfinder, &InstanceHolder)>,
    ) {
        for event in pearl_goto_events.read().cloned() {
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

            pearl_pull_events.send(PearlPullEvent(event.0));
        }
    }

    pub fn handle_pull_pearl_events(
        mut goto_events: EventWriter<GotoEvent>,
        mut pearl_pending_events: ResMut<ResendPearlEvents>,
        mut pearl_pull_events: EventReader<PearlPullEvent>,
        mut send_packet_events: EventWriter<SendPacketEvent>,
        mut query: Query<(&Pathfinder, &TabList)>,
    ) {
        for event in pearl_pull_events.read().cloned() {
            let Ok((pathfinder, tab_list)) = query.get_mut(event.entity) else {
                continue;
            };

            if let Some(_goal) = &pathfinder.goal {
                pearl_pending_events.pull.push(event);
                continue;
            }

            if !tab_list.contains_key(&event.owner_uuid) {
                pearl_pending_events.goto.push(PearlGotoEvent(event.0));
                continue;
            }

            let packet = ServerboundGamePacket::UseItemOn(ServerboundUseItemOn {
                hand:      InteractionHand::MainHand,
                block_hit: BlockHit {
                    block_pos:    event.block_pos,
                    direction:    Direction::Down,
                    location:     Vec3 {
                        x: f64::from(event.block_pos.x) + 0.5,
                        y: f64::from(event.block_pos.y) + 0.5,
                        z: f64::from(event.block_pos.z) + 0.5,
                    },
                    inside:       false,
                    world_border: false,
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
}
