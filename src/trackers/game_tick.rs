use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
};
use bounded_counter::BoundedCounter;

pub struct GameTickPlugin;

impl Plugin for GameTickPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTicks::default())
            .add_systems(GameTick, Self::handle_game_ticks);
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut, Resource)]
pub struct GameTicks(BoundedCounter<u128>);

type InitQueryData = Entity;
type InitQueryFilter = (With<LocalEntity>, With<Player>, Without<GameTicks>);

type RunQueryData<'a> = &'a mut GameTicks;
type RunQueryFilter = (With<LocalEntity>, With<Player>, With<GameTicks>);

impl GameTickPlugin {
    pub fn handle_game_ticks(
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
}
