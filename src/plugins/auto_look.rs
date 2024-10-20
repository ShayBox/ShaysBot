use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, EyeHeight, LocalEntity, Position},
    nearest_entity::EntityFinder,
    pathfinder::tick_execute_path,
    physics::travel,
    prelude::*,
    LookAtEvent,
};

pub struct AutoLookPlugin;

impl Plugin for AutoLookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            handle_auto_look.after(travel).after(tick_execute_path),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_auto_look(
    query: Query<Entity, (With<LocalEntity>, With<Player>)>,
    entities: EntityFinder,
    targets: Query<(&Position, Option<&EyeHeight>)>,
    mut look_at_events: EventWriter<LookAtEvent>,
) {
    for entity in query.iter() {
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
