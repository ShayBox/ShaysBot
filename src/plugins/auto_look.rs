use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{clamp_look_direction, metadata::Player, EyeHeight, LocalEntity, Position},
    nearest_entity::EntityFinder,
    physics::PhysicsSet,
    prelude::GameTick,
    LookAtEvent,
};

pub struct AutoLookPlugin;

impl Plugin for AutoLookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            handle_auto_look
                .after(clamp_look_direction)
                .before(PhysicsSet),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_auto_look(
    mut query: Query<Entity, (With<LocalEntity>, With<Player>)>,
    entities: EntityFinder<With<Player>>,
    targets: Query<(&Position, Option<&EyeHeight>)>,
    mut look_at_events: EventWriter<LookAtEvent>,
) {
    for entity in &mut query {
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

        look_at_events.send(LookAtEvent { entity, position });
    }
}
