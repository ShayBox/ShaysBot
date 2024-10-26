use std::sync::Arc;

use azalea::{
    app::{App, First, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
};
use derive_config::DeriveTomlConfig;
use parking_lot::RwLock;

use crate::Settings;

pub struct SettingsPlugin(pub Arc<RwLock<Settings>>);

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, load_settings);
    }
}

#[derive(Component, Clone)]
pub struct SettingsLock(pub Arc<RwLock<Settings>>);

impl SettingsLock {
    fn new(settings: Settings) -> Self {
        Self(Arc::new(RwLock::new(settings)))
    }
}

type QueryData = Entity;
type QueryFilter = (With<Player>, With<LocalEntity>, Without<SettingsLock>);

/// # Panics
/// Will panic if `Settings::save` fails.
pub fn load_settings(mut query: Query<QueryData, QueryFilter>, mut commands: Commands) {
    let Ok(entity) = query.get_single_mut() else {
        return;
    };

    /* TODO: Separate file for each swarm instance */
    let settings = Settings::load().unwrap_or_default();
    settings.save().expect("Failed to save settings");

    /* TODO: Remove lock once ClientExt is gone */
    commands.entity(entity).insert(SettingsLock::new(settings));
}

pub trait SettingsClientExt {
    fn get_settings(&self) -> SettingsLock;
}

impl SettingsClientExt for Client {
    fn get_settings(&self) -> SettingsLock {
        self.ecs
            .lock()
            .get::<SettingsLock>(self.entity)
            .expect("Missing SettingsLock Component")
            .clone()
    }
}
