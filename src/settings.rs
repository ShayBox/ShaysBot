use std::sync::Arc;

use azalea::{
    app::{App, First, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
    protocol::ServerAddress,
};
use derive_config::DeriveTomlConfig;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, DeriveTomlConfig, Deserialize, Serialize)]
pub struct Settings {
    /// This is the distance in blocks that ender pearls are visible from the player.
    /// It is better to under-estimate the value than to over-estimate it.
    /// If you notice pearls not saving outside visual range, try decreasing this value.
    /// If you notice manually pulled pearls not being removed, try increasing this value.
    pub pearl_view_distance: i32,

    /// Minecraft Server Address
    pub server_address: ServerAddress,

    /// Minecraft Chat Prefix
    pub chat_prefix: String,

    /// Minecraft Account Username
    pub username: String,

    /// Minecraft Auth Mode
    pub online: bool,

    /// Quiet Mode
    pub quiet: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pearl_view_distance: 64,
            server_address: ServerAddress::try_from("play.vengeancecraft.net").unwrap(),
            chat_prefix: String::from("!"),
            username: String::from("ShaysBot"),
            online: true,
            quiet: false,
        }
    }
}
