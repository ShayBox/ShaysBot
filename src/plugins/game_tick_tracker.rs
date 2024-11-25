use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
};
use bounded_counter::BoundedCounter;

pub struct GameTickTrackerPlugin;

impl Plugin for GameTickTrackerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTicks::default())
            .add_systems(GameTick, handle_game_tick);
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut, Resource)]
pub struct GameTicks(BoundedCounter<u128>);

type InitQueryData = Entity;
type InitQueryFilter = (With<LocalEntity>, With<Player>, Without<GameTicks>);

type RunQueryData<'a> = &'a mut GameTicks;
type RunQueryFilter = (With<LocalEntity>, With<Player>, With<GameTicks>);

pub fn handle_game_tick(
    mut init_query: Query<InitQueryData, InitQueryFilter>,
    mut commands: Commands,

    mut run_query: Query<RunQueryData, RunQueryFilter>,
    mut game_ticks: ResMut<GameTicks>,
) {
    for entity in &mut init_query {
        commands.entity(entity).insert(GameTicks::default());
    }

    game_ticks.next(); /* Global game ticks counter */
    for mut game_ticks in &mut run_query {
        game_ticks.next(); /* Local game ticks counter */
    }
}
