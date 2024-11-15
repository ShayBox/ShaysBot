use std::{collections::HashMap, time::Duration};

use azalea::{
    app::{App, Plugin},
    prelude::*,
    Vec3,
};
use derive_config::DeriveTomlConfig;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, DurationSeconds, NoneAsEmptyString};
use serenity::all::ChannelId;
use smart_default::SmartDefault;
use uuid::Uuid;

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum EncryptionMode {
    #[default]
    OnDemand,
    Always,
    Never,
}

strike! {
    #[strikethrough[serde_as]]
    #[strikethrough[allow(clippy::derive_partial_eq_without_eq)]]
    #[strikethrough[derive(Clone, PartialEq, Deserialize, Serialize, SmartDefault)]]
    #[strikethrough[serde(default)]]
    #[derive(DeriveTomlConfig, Resource)]
    pub struct Settings {
        /// Minecraft server ender pearl view distance in blocks.
        /// It is better to under-estimate than to over-estimate.
        #[default(64)]
        pub pearl_view_distance: i32,

        /// Minecraft online-mode auth.
        #[default(true)]
        pub online_mode: bool,

        /// Minecraft account username.
        #[default("ShaysBot")]
        pub account_username: String,

        /// Minecraft server address.
        #[default("play.vengeancecraft.net")]
        pub server_address: String,

        /// Chat command prefix.
        #[default("!")]
        pub command_prefix: String,

        /// Command cooldown in seconds.
        #[default(Duration::from_secs(10))]
        #[serde_as(as = "DurationSeconds")]
        pub command_cooldown: Duration,

        /// Discord client token for commands and events. (Optional)
        #[default("")]
        pub discord_token: String,

        /// Discord channel id for events. (Optional)
        #[serde_as(as = "DisplayFromStr")]
        pub discord_channel: ChannelId,

        /// Target location to idle at after pearling.
        pub idle: pub struct IdleGoal {
            #[serde_as(as = "DisplayFromStr")]
            pub pos: Vec3,
            pub radius: f32,
        },

        /// Chat encryption using the NCR (No Chat Reports) mod.
        pub encryption: pub struct ChatEncryption {
            /// Encryption key (default is public)
            #[default("blfrngArk3chG6wzncOZ5A==")]
            pub key: String,

            /// Encryption response mode. (OnDemand, Always, or Never)
            #[default(EncryptionMode::OnDemand)]
            pub mode: EncryptionMode,
        },

        /// Disable in-game command responses globally.
        #[default(false)]
        pub disable_responses: bool,

        /// Automatically disconnect and exit when an un-whitelisted player enters visual range.
        #[default(false)]
        pub unknown_player_auto_exit: bool,

        // Disable in-game commands for non-whitelisted players.
        // pub whitelist: bool, /* TODO: 0.8.0 */

        /// Minecraft and Discord users allowed to use the bot.
        #[serde_as(as = "HashMap<_, NoneAsEmptyString>")]
        #[serde(alias = "whitelist")] /* Deprecated: 0.7.2 */
        pub whitelisted: HashMap<Uuid, Option<String>>,
    }
}

impl Plugin for Settings {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone());
    }
}
