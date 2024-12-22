use azalea::{
    app::{App, Plugin},
    attack::AttackEvent,
    ecs::prelude::*,
    entity::{metadata::AbstractMonster, EyeHeight, Position},
    nearest_entity::EntityFinder,
    prelude::*,
    world::MinecraftEntityId,
    LookAtEvent,
};

use crate::prelude::{GameTickPlugin, GameTicks, LocalSettings};

/// Automatically attack the closest monster
pub struct AutoKillPlugin;

impl Plugin for AutoKillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            Self::handle_auto_kill.before(GameTickPlugin::handle_game_ticks),
        );
    }
}

impl AutoKillPlugin {
    pub fn handle_auto_kill(
        mut query: Query<(Entity, &GameTicks, &LocalSettings)>,
        entities: EntityFinder<With<AbstractMonster>>,
        targets: Query<(&Position, Option<&EyeHeight>, &MinecraftEntityId)>,
        mut look_at_events: EventWriter<LookAtEvent>,
        mut attack_events: EventWriter<AttackEvent>,
    ) {
        for (entity, game_ticks, local_settings) in &mut query {
            if game_ticks.0 % local_settings.auto_kill.delay_ticks != 0 {
                continue;
            }

            let Some(target) = entities.nearest_to_entity(entity, 4.20) else {
                continue;
            };

            let Ok((target_pos, target_eye_height, entity_id)) = targets.get(target) else {
                continue;
            };

            let mut position = **target_pos;
            if let Some(eye_height) = target_eye_height {
                position.y += f64::from(**eye_height);
            }

            look_at_events.send(LookAtEvent { entity, position });
            attack_events.send(AttackEvent {
                entity,
                target: *entity_id,
            });
        }
    }
}
