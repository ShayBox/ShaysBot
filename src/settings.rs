use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    prelude::*,
    protocol::ServerAddress,
};
use derive_config::DeriveTomlConfig;
use serde::{Deserialize, Serialize};

pub struct SettingsPlugin(pub Settings);

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone());
    }
}

#[derive(Clone, Debug, DeriveTomlConfig, Deserialize, Serialize, Resource)]
pub struct Settings {
    /// This is the distance in blocks that ender pearls are visible from the player.
    /// It is better to under-estimate the value than to over-estimate it.
    /// If you notice pearls not saving outside visual range, try decreasing this value.
    /// If you notice manually pulled pearls not being removed, try increasing this value.
    pub pearl_view_distance: i32,

    /// Minecraft Server Address
    pub server_address: ServerAddress,

    // Discord Client Token
    pub discord_token: Option<String>,

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
            discord_token: None,
            chat_prefix: String::from("!"),
            username: String::from("ShaysBot"),
            online: true,
            quiet: false,
        }
    }
}
