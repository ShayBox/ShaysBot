use azalea::{
    BlockPos,
    Vec3,
    app::{App, Plugin, Update},
    chunks::handle_receive_chunk_event,
    core::direction::Direction,
    ecs::prelude::*,
    interact::handle_swing_arm_trigger,
    inventory::InventorySystems,
    local_player::TabList,
    mining::MiningSystems,
    packet::game::{SendGamePacketEvent, handle_outgoing_packets_observer},
    pathfinder::{GotoEvent, Pathfinder, PathfinderOpts, goals::RadiusGoal, goto_listener},
    physics::PhysicsSystems,
    prelude::*,
    protocol::packets::game::{
        ServerboundGamePacket,
        ServerboundUseItemOn,
        s_interact::InteractionHand,
        s_use_item_on::BlockHit,
    },
};
use uuid::Uuid;

use crate::prelude::*;

/// Automatically goto and pull player stasis chambers.
pub struct AutoPearlPlugin;

impl Plugin for AutoPearlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ResendPearlEvents::default())
            .add_message::<PearlGotoEvent>()
            .add_message::<PearlPullEvent>()
            .add_systems(
                Update,
                (
                    Self::handle_resend_pearl_events,
                    Self::handle_goto_pearl_events
                        .before(goto_listener)
                        .after(handle_receive_chunk_event),
                    Self::handle_pull_pearl_events
                        .before(handle_outgoing_packets_observer)
                        .before(handle_swing_arm_trigger)
                        .after(InventorySystems)
                        .after(PhysicsSystems)
                        .after(MiningSystems),
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Debug)]
pub struct PearlEvent {
    pub entity:     Entity,
    pub idle_goal:  IdleGoal,
    pub block_pos:  BlockPos,
    pub owner_uuid: Uuid,
}

#[derive(Clone, Debug, Deref, DerefMut, Message)]
pub struct PearlGotoEvent(pub PearlEvent);

#[derive(Clone, Debug, Deref, DerefMut, Message)]
pub struct PearlPullEvent(pub PearlEvent);

#[derive(Default, Resource)]
pub struct ResendPearlEvents {
    goto: Vec<PearlGotoEvent>,
    pull: Vec<PearlPullEvent>,
}

impl AutoPearlPlugin {
    pub fn handle_resend_pearl_events(
        mut pearl_goto_events: MessageWriter<PearlGotoEvent>,
        mut pearl_pull_events: MessageWriter<PearlPullEvent>,
        mut pearl_pending_events: ResMut<ResendPearlEvents>,
        mut query: Query<&Pathfinder>,
    ) {
        for pathfinder in &mut query {
            if let Some(_goal) = &pathfinder.goal {
                continue;
            }

            if let Some(event) = pearl_pending_events.goto.pop() {
                debug!("Resending {event:#?}");
                pearl_goto_events.write(event);
            }

            if let Some(event) = pearl_pending_events.pull.pop() {
                debug!("Resending {event:#?}");
                pearl_pull_events.write(event);
            }
        }
    }

    pub fn handle_goto_pearl_events(
        mut goto_events: MessageWriter<GotoEvent>,
        mut pearl_goto_events: MessageReader<PearlGotoEvent>,
        mut pearl_pull_events: MessageWriter<PearlPullEvent>,
        mut pearl_pending_events: ResMut<ResendPearlEvents>,
        mut query: Query<&Pathfinder>,
    ) {
        for event in pearl_goto_events.read().cloned() {
            let Ok(pathfinder) = query.get_mut(event.entity) else {
                continue;
            };

            if let Some(_goal) = &pathfinder.goal {
                pearl_pending_events.goto.push(event.clone());
                continue;
            }

            let pos = event.block_pos.to_vec3_floored();
            goto_events.write(GotoEvent::new(
                event.entity,
                RadiusGoal { radius: 3.0, pos },
                PathfinderOpts::default()
                    .allow_mining(false)
                    .retry_on_no_path(false),
            ));

            pearl_pull_events.write(PearlPullEvent(event.0));
        }
    }

    pub fn handle_pull_pearl_events(
        mut goto_events: MessageWriter<GotoEvent>,
        mut pearl_pending_events: ResMut<ResendPearlEvents>,
        mut pearl_pull_events: MessageReader<PearlPullEvent>,
        mut commands: Commands,
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
                seq:       0,
            });

            commands.trigger(SendGamePacketEvent {
                sent_by: event.entity,
                packet,
            });

            let goal = RadiusGoal {
                pos:    event.idle_goal.coords,
                radius: event.idle_goal.radius + 1.0,
            };
            if event.idle_goal != IdleGoal::default() {
                goto_events.write(GotoEvent::new(
                    event.entity,
                    goal,
                    PathfinderOpts::default()
                        .allow_mining(false)
                        .retry_on_no_path(false),
                ));
            }
        }
    }
}
