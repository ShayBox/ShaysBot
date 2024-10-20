use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    interact::{handle_swing_arm_event, SwingArmEvent}
    ,
    prelude::*,
};

pub struct AntiAfkPlugin;

impl Plugin for AntiAfkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(GameTick, handle_anti_afk.before(handle_swing_arm_event));
    }
}

#[derive(Component, Default)]
struct AntiAfkState {
    ticks: u8,
}

#[allow(clippy::type_complexity)]
fn handle_anti_afk(
    mut query: Query<(&mut AntiAfkState, Entity), (With<Player>, With<LocalEntity>)>,
    mut swing_arm_events: EventWriter<SwingArmEvent>,
) {
    for (mut state, entity) in &mut query.iter_mut() {
        if state.ticks == u8::MAX {
            state.ticks = u8::MIN;

            swing_arm_events.send(SwingArmEvent { entity });
        }

        state.ticks += 1;
    }
}

pub trait AntiAfkClientExt {
    fn toggle_anti_afk(&self);
}

impl AntiAfkClientExt for Client {
    fn toggle_anti_afk(&self) {
        let mut ecs = self.ecs.lock();
        let mut entity = ecs.entity_mut(self.entity);

        if let Some(_state) = entity.get::<AntiAfkState>() {
            entity.remove::<AntiAfkState>();
        } else {
            entity.insert(AntiAfkState::default());
        }
    }
}
