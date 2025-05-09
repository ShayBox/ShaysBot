use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, EyeHeight, Position},
    nearest_entity::EntityFinder,
    physics::PhysicsSet,
    prelude::*,
    LookAtEvent,
};

use crate::prelude::{GameTicks, LocalSettings};

/// Automatically look at the closest player in range
pub struct AutoLookPlugin;

impl Plugin for AutoLookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(GameTick, Self::handle_auto_look.before(PhysicsSet));
    }
}

impl AutoLookPlugin {
    pub fn handle_auto_look(
        mut query: Query<(Entity, &GameTicks, &LocalSettings)>,
        entities: EntityFinder<With<Player>>,
        targets: Query<(&Position, Option<&EyeHeight>)>,
        mut look_at_events: EventWriter<LookAtEvent>,
    ) {
        for (entity, game_ticks, local_settings) in &mut query {
            if !local_settings.auto_look.enabled {
                continue;
            }

            if game_ticks.0 % local_settings.auto_look.delay_ticks != 0 {
                continue;
            }

            let Some(target) = entities.nearest_to_entity(entity, f64::MAX) else {
                continue;
            };

            let Ok((target_pos, target_eye_height)) = targets.get(target) else {
                continue;
            };

            let mut position = **target_pos;
            if let Some(eye_height) = target_eye_height {
                position.y += f64::from(**eye_height);
            }

            look_at_events.write(LookAtEvent { entity, position });
        }
    }
}
