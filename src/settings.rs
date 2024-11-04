use azalea::{
    app::{App, Plugin},
    prelude::*,
};
use derive_config::DeriveTomlConfig;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum EncryptionMode {
    #[default]
    OnDemand,
    Always,
    Never,
}

#[derive(Clone, Deserialize, Serialize, SmartDefault)]
pub struct ChatEncryption {
    /// `OnDemand` | `Always` | `Never`
    #[default(EncryptionMode::OnDemand)]
    pub mode: EncryptionMode,

    /// Default No Chat Reports mod key
    #[default("blfrngArk3chG6wzncOZ5A==")]
    pub key: String,
}

#[derive(Clone, DeriveTomlConfig, Deserialize, Resource, Serialize, SmartDefault)]
#[serde(default)] /* Default new or missing fields instead of the whole struct */
pub struct Settings {
    /// This is the distance in blocks that ender pearls are visible from the player.
    /// It is better to under-estimate the value than to over-estimate it.
    /// If you notice pearls not saving outside visual range, try decreasing this value.
    /// If you notice manually pulled pearls not being removed, try increasing this value.
    #[default(64)]
    pub pearl_view_distance: i32,

    /// Minecraft Server Address
    #[default("play.vengeancecraft.net")]
    pub server_address: String,

    /// Discord Client Token (Optional)
    #[default("")]
    pub discord_token: String,

    /// Minecraft Chat Prefix
    #[default("!")]
    pub chat_prefix: String,

    /// Minecraft Account Username
    #[default("ShaysBot")]
    pub username: String,

    /// Minecraft Online Auth
    #[default(true)]
    pub online: bool,

    /// Disable in-game command responses
    #[default(false)]
    pub quiet: bool,

    /// Minecraft Encryption Mode (NCR Mod)
    pub encryption: ChatEncryption,
}

impl Plugin for Settings {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone());
    }
}
