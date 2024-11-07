use std::collections::HashMap;

use azalea::{
    app::{App, Plugin},
    prelude::*,
};
use derive_config::DeriveTomlConfig;
use serde::{Deserialize, Serialize};
use serde_with::NoneAsEmptyString;
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
    #[strikethrough[derive(Clone, Deserialize, Serialize, SmartDefault)]]
    #[strikethrough[serde(default)]]
    #[derive(DeriveTomlConfig, Resource)]
    pub struct Settings {
        /// Minecraft server ender pearl view distance in blocks.
        /// It is better to under-estimate than to over-estimate.
        #[default(64)]
        pub pearl_view_distance: i32,

        /// Disable in-game command responses globally.
        #[default(false)]
        #[serde(alias = "quiet")] /* Deprecated: 0.6.0 */
        pub disable_responses: bool,

        /// Minecraft online-mode auth.
        #[default(true)]
        #[serde(alias = "online")] /* Deprecated: 0.6.0 */
        pub online_mode: bool,

        /// Minecraft account username.
        #[default("ShaysBot")]
        #[serde(alias = "username")] /* Deprecated: 0.6.0 */
        pub account_username: String,

        /// Minecraft server address.
        #[default("play.vengeancecraft.net")]
        pub server_address: String,

        /// Discord client token. (Optional)
        #[default("")]
        pub discord_token: String,

        /// Chat command prefix.
        #[default("!")]
        #[serde(alias = "chat_prefix")] /* Deprecated: 0.6.0 */
        pub command_prefix: String,

        /// Chat encryption using the NCR (No Chat Reports) mod.
        pub encryption: pub struct ChatEncryption {
            /// Encryption key (default is public)
            #[default("blfrngArk3chG6wzncOZ5A==")]
            pub key: String,

            /// Encryption response mode. (OnDemand, Always, or Never)
            #[default(EncryptionMode::OnDemand)]
            pub mode: EncryptionMode,
        },

        /// Minecraft and Discord users allowed to use the bot.
        /// The whitelist is disabled if it's empty.
        #[serde_as(as = "HashMap<_, NoneAsEmptyString>")]
        pub whitelist: HashMap<Uuid, Option<String>>,
    }
}

impl Plugin for Settings {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone());
    }
}
