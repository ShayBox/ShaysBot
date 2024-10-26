use std::sync::Arc;

use azalea::{
    app::{App, First, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
};
use derive_config::DeriveYamlConfig;
use parking_lot::RwLock;

use crate::Trapdoors;

pub struct TrapdoorsPlugin(pub Arc<RwLock<Trapdoors>>);

impl Plugin for TrapdoorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, load_trapdoors);
    }
}

#[derive(Component, Clone)]
pub struct TrapdoorsLock(pub Arc<RwLock<Trapdoors>>);

impl TrapdoorsLock {
    fn new(trapdoors: Trapdoors) -> Self {
        Self(Arc::new(RwLock::new(trapdoors)))
    }
}

type QueryData = Entity;
type QueryFilter = (With<Player>, With<LocalEntity>, Without<TrapdoorsLock>);

/// # Panics
/// Will panic if `Trapdoors::save` fails.
pub fn load_trapdoors(mut query: Query<QueryData, QueryFilter>, mut commands: Commands) {
    let Ok(entity) = query.get_single_mut() else {
        return;
    };

    /* TODO: Separate file for each swarm instance */
    let trapdoors = Trapdoors::load().unwrap_or_default();
    trapdoors.save().expect("Failed to save settings");

    /* TODO: Remove lock once ClientExt is gone */
    commands
        .entity(entity)
        .insert(TrapdoorsLock::new(trapdoors));
}

pub trait TrapdoorsClientExt {
    fn get_trapdoors(&self) -> TrapdoorsLock;
}

impl TrapdoorsClientExt for Client {
    fn get_trapdoors(&self) -> TrapdoorsLock {
        self.ecs
            .lock()
            .get::<TrapdoorsLock>(self.entity)
            .expect("Missing TrapdoorsLock Component")
            .clone()
    }
}
