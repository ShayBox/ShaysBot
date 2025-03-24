use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    interact::{handle_swing_arm_event, SwingArmEvent},
    mining::continue_mining_block,
    prelude::*,
    ClientInformation,
};

use crate::prelude::*;

/// Automatically swing arm to avoid being kicked
pub struct AntiAfkPlugin;

impl Plugin for AntiAfkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            Self::handle_anti_afk
                .before(handle_swing_arm_event)
                .after(continue_mining_block)
                .after(GameTickPlugin::handle_game_ticks),
        );
    }
}

impl AntiAfkPlugin {
    pub fn handle_anti_afk(
        mut query: Query<(Entity, &LocalSettings, &GameTicks, &mut ClientInformation)>,
        mut swing_arm_events: EventWriter<SwingArmEvent>,
    ) {
        for (entity, local_settings, game_ticks, mut client_information) in &mut query {
            if !local_settings.anti_afk.enabled {
                continue;
            }

            if game_ticks.0 % local_settings.anti_afk.delay_ticks != 0 {
                continue;
            }

            trace!("Anti-Afk Swing Arm Event");
            client_information.view_distance = 2;
            swing_arm_events.send(SwingArmEvent { entity });
        }
    }
}
