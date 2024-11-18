use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    interact::{handle_swing_arm_event, SwingArmEvent},
    mining::continue_mining_block,
    prelude::*,
};

use crate::BoundedCounter;

/// Automatically send swing arm packets every 256 ticks
pub struct AntiAfkPlugin;

impl Plugin for AntiAfkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            handle_anti_afk
                .before(handle_swing_arm_event)
                .after(continue_mining_block),
        );
    }
}

#[derive(Component, Default)]
pub struct AntiAfkCounter(BoundedCounter<u8>);

type InitQueryData = Entity;
type InitQueryFilter = (With<Player>, With<LocalEntity>, Without<AntiAfkCounter>);

type RunQueryData<'a> = (Entity, &'a mut AntiAfkCounter);
type RunQueryFilter = (With<Player>, With<LocalEntity>, With<AntiAfkCounter>);

pub fn handle_anti_afk(
    mut init_query: Query<InitQueryData, InitQueryFilter>,
    mut commands: Commands,

    mut run_query: Query<RunQueryData, RunQueryFilter>,
    mut swing_arm_events: EventWriter<SwingArmEvent>,
) {
    for entity in &mut init_query {
        commands.entity(entity).insert(AntiAfkCounter::default());
    }

    for (entity, mut counter) in &mut run_query {
        let Some(ticks) = counter.0.next() else {
            return;
        };

        if ticks == u8::MAX {
            swing_arm_events.send(SwingArmEvent { entity });
        }
    }
}
