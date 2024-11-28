use std::{collections::HashMap, time::Duration};

use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
    protocol::ServerAddress,
    GameProfileComponent,
    Vec3,
};
use derive_config::DeriveTomlConfig;
use derive_new::new as New;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, DurationSeconds, NoneAsEmptyString};
use serenity::all::ChannelId;
use smart_default::SmartDefault;
use uuid::Uuid;

#[serde_as]
#[derive(Clone, DeriveTomlConfig, Deserialize, Serialize, SmartDefault, Resource)]
#[serde(default)]
pub struct Settings {
    /// Minecraft server ender pearl view distance in blocks.
    /// It is better to under-estimate than to over-estimate.
    #[default(64)]
    pub pearl_view_distance: i32,

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

    /// Minecraft server address.
    #[default(ServerAddress{
        host: str!("play.vengeancecraft.net"),
        port: 25565
    })]
    pub server_address: ServerAddress,

    /// Automatically disconnect and exit when an un-whitelisted player enters visual range.
    #[default(false)]
    pub unknown_player_auto_exit: bool,

    /// Disable in-game commands for non-whitelisted players.
    #[default(false)]
    pub whitelist: bool,

    /// Chat encryption using the NCR (No Chat Reports) mod.
    pub encryption: ChatEncryption,

    /// Minecraft bot accounts
    #[default(HashMap::from([
        (str!("primary"), BotSettings::new(str!("Test"))),
        #[cfg(debug_assertions)]
        (str!("secondary"), BotSettings::new(str!("Fishdar"))),
    ]))]
    #[serde(rename = "location")]
    pub locations: HashMap<String, BotSettings>,

    /// Whitelisted Minecraft accounts and their linked Discord accounts.
    #[serde_as(as = "HashMap<_, NoneAsEmptyString>")]
    pub whitelisted: HashMap<Uuid, Option<String>>,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, SmartDefault)]
pub struct ChatEncryption {
    /// Encryption key (default is public)
    #[default("blfrngArk3chG6wzncOZ5A==")]
    pub key: String,

    /// Encryption response mode. (`OnDemand`, `Always`, or `Never`)
    #[default(EncryptionMode::OnDemand)]
    pub mode: EncryptionMode,
}

#[derive(Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub enum EncryptionMode {
    #[default]
    OnDemand,
    Always,
    Never,
}

#[serde_as]
#[derive(Clone, Component, Deserialize, Serialize, SmartDefault, New)]
#[serde(default)]
pub struct BotSettings {
    /// Minecraft Online Mode.
    #[default(true)]
    #[new(value = "true")]
    pub online_mode: bool,

    /// Minecraft Account Username.
    pub account_username: String,

    /// Minecraft Server Address. (Optional)
    /// This must be a proxy to the same server. (ZenithProxy / ViaProxy)
    #[default(None)]
    #[new(value = "None")]
    pub server_address: Option<ServerAddress>,

    /// Discord Channel ID. (Optional)
    #[new(default)]
    pub discord_channel: ChannelId,

    /// Disable in-game command responses.
    #[default(false)]
    #[new(value = "false")]
    pub disable_responses: bool,

    /// Idle Goal. (after pearling)
    #[new(default)]
    #[serde(rename = "idle")]
    pub idle_goal: IdleGoal,
}

#[serde_as]
#[derive(Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct IdleGoal {
    #[serde_as(as = "DisplayFromStr")]
    pub coords: Vec3,
    pub radius: f32,
}

impl Plugin for Settings {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_systems(Update, handle_init_bot_settings);
    }
}

type QueryData<'a> = (Entity, &'a GameProfileComponent);
type QueryFilter = (With<LocalEntity>, With<Player>, Without<BotSettings>);

fn handle_init_bot_settings(
    query: Query<QueryData, QueryFilter>,
    settings: Res<Settings>,
    mut commands: Commands,
) {
    for (entity, profile) in &query {
        if let Some(bot_settings) = settings
            .locations
            .values()
            .find(|bs| bs.account_username == profile.name)
        {
            commands.entity(entity).insert(bot_settings.clone());
        }
    }
}
