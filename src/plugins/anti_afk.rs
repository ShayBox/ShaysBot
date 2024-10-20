use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    interact::{handle_swing_arm_event, SwingArmEvent},
    mining::continue_mining_block,
    prelude::*,
};

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

#[derive(Default)]
struct Ticks(u8);

impl Iterator for Ticks {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 += 1;
        self.0 %= u8::MAX;

        Some(self.0)
    }
}

#[derive(Component, Default)]
struct Afk {
    ticks: Ticks,
}

#[allow(clippy::type_complexity)]
fn handle_anti_afk(
    mut query: Query<(&mut Afk, Entity), (With<Player>, With<LocalEntity>)>,
    mut swing_arm_events: EventWriter<SwingArmEvent>,
) {
    for (mut afk, entity) in &mut query.iter_mut() {
        let Some(ticks) = afk.ticks.next() else {
            return;
        };

        if ticks == u8::MAX {
            swing_arm_events.send(SwingArmEvent { entity });
        }
    }
}

pub trait AntiAfkClientExt {
    fn init_anti_afk(&self);
}

impl AntiAfkClientExt for Client {
    fn init_anti_afk(&self) {
        let mut ecs = self.ecs.lock();
        let mut entity = ecs.entity_mut(self.entity);

        if entity.get::<Afk>().is_none() {
            entity.insert(Afk::default());
        }
    }
}
