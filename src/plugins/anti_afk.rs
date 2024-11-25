use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    interact::{handle_swing_arm_event, SwingArmEvent},
    mining::continue_mining_block,
    prelude::*,
};

use crate::{handle_game_tick, plugins::game_tick_tracker::GameTicks};

/// Automatically send swing arm packets every 25k ticks
pub struct AntiAfkPlugin;

impl Plugin for AntiAfkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            handle_anti_afk
                .before(handle_swing_arm_event)
                .after(continue_mining_block)
                .after(handle_game_tick),
        );
    }
}

pub fn handle_anti_afk(
    query: Query<(Entity, &GameTicks)>,
    mut swing_arm_events: EventWriter<SwingArmEvent>,
) {
    for (entity, game_ticks) in query.iter() {
        if game_ticks.0 % 25_000 == 0 {
            debug!("Anti-Afk Swing Arm Event");
            swing_arm_events.send(SwingArmEvent { entity });
        }
    }
}
