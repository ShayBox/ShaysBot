use std::{
    fs::File,
    io::{ErrorKind, Read, Seek, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use azalea::{
    app::{App, Plugin, Startup},
    ecs::prelude::*,
    prelude::*,
    protocol::{address::ServerAddr, resolve},
    swarm::Swarm,
    JoinOpts,
    NoState,
    Vec3,
};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
#[cfg(feature = "bot")]
use serenity::all::ChannelId;
use smart_default::SmartDefault;

/// Local Account Settings that apply per-account
pub struct LocalSettingsPlugin;

impl Plugin for LocalSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::handle_startup);
    }
}

impl LocalSettingsPlugin {
    pub fn handle_startup(swarm: Res<Swarm>) {
        let swarm = swarm.clone();

        tokio::task::spawn_local(async move {
            if let Err(error) = load_settings(swarm).await {
                error!("There was an error loading local settings: {error}");
                std::process::exit(1);
            }
        });
    }
}

#[derive(Clone, Component, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct LocalSettings {
    #[serde(skip)]
    path: PathBuf,

    /// Minecraft account authentication mode.
    pub auth_mode: AuthMode,

    /// Anti Afk module settings.
    pub anti_afk: AntiAfk,

    /// Anti Spam module settings.
    pub anti_spam: AntiSpam,

    /// Auto eat module settings.
    pub auto_eat: AutoEat,

    /// Auto kill module settings.
    pub auto_kill: AutoKill,

    /// Auto leave module settings.
    pub auto_leave: AutoLeave,

    /// Auto look module settings.
    pub auto_look: AutoLook,

    /// Auto pearl module settings.
    pub auto_pearl: AutoPearl,

    /// Disable in-game command responses.
    pub disable_responses: bool,

    /// Discord Channel ID. (Optional)
    #[cfg(feature = "bot")]
    pub discord_channel: ChannelId,

    /// Minecraft account server address. (Optional)
    pub server_address: Option<ServerAddr>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    #[default]
    Offline,
    Online,
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct AntiAfk {
    #[default(true)]
    pub enabled: bool,

    #[default(1234)]
    #[serde_as(as = "DisplayFromStr")]
    pub delay_ticks: u128,

    #[default(12)]
    pub view_distance: u8,
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct AntiSpam {
    #[default(true)]
    pub enabled: bool,

    #[default(true)]
    pub unix_epoch: bool,
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct AutoEat {
    #[default(true)]
    pub enabled: bool,

    #[default(42)]
    #[serde_as(as = "DisplayFromStr")]
    pub delay_ticks: u128,
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct AutoKill {
    #[default(false)]
    pub enabled: bool,

    #[default(true)]
    pub auto_weapon: bool,

    #[default(25)]
    #[serde_as(as = "DisplayFromStr")]
    pub delay_ticks: u128,
}

#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct AutoLeave {
    /// Automatically leave the server if an unknown player enters visual range
    #[default(false)]
    pub unknown_player: bool,

    /// Stay disconnected if `ZenithProxy` disconnects us and force disconnect through `ZenithProxy`
    #[default(false)]
    pub grim_disconnect: bool,

    /// Automatically leave to re-queue on 2B2T when another bot at the same location enters visual range
    #[default(false)]
    pub auto_requeue: bool,
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct AutoLook {
    #[default(true)]
    pub enabled: bool,

    #[default(2)]
    #[serde_as(as = "DisplayFromStr")]
    pub delay_ticks: u128,
}

#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct AutoPearl {
    #[default(true)]
    pub enabled: bool,

    /// Location name
    pub location: String,

    /// Pearl limit for each player.
    #[default(10)]
    pub pearl_limit: usize,

    /// Position & Radius to idle after pearling.
    #[serde(rename = "idle")]
    pub idle_goal: IdleGoal,
}

#[serde_as]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct IdleGoal {
    #[serde_as(as = "DisplayFromStr")]
    pub coords: Vec3,
    pub radius: f32,
}

impl From<PathBuf> for LocalSettings {
    fn from(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }
}

impl LocalSettings {
    /// # Errors
    /// Will return `Err` if `Self::path` fails.
    pub fn new(username: &str) -> Result<Self> {
        Ok(Self::from(Self::path()?.join(format!("{username}.toml"))))
    }

    /// # Errors
    /// Will return `Err` if `std::env::current_exe` or `std::env::current_dir` fails.
    pub fn path() -> Result<PathBuf> {
        let path = if cfg!(debug_assertions) {
            let path = std::env::current_exe()?;
            path.parent().context("None")?.to_path_buf()
        } else {
            std::env::current_dir()?
        };

        Ok(path.join("local-settings"))
    }

    /// # Errors
    /// Will return `Err` if `File::open`, `toml::to_string_pretty`, or `File::write_all` fails.
    pub fn load(self) -> Result<Self> {
        match File::open(&self.path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Self::from(self.path).save(),
            Err(error) => bail!(error),
            Ok(mut file) => {
                let mut text = String::new();
                file.read_to_string(&mut text)?;
                file.rewind()?;

                let mut local_settings = toml::from_str::<Self>(&text)?;
                local_settings.path = self.path; /* Fix serde replacing path */

                Ok(local_settings)
            }
        }
    }

    /// # Errors
    /// Will return `Err` if `File::open`, `File::read_to_string`, `File::rewind`, or `toml::from_str` fails.
    pub fn save(self) -> Result<Self> {
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;

        let text = toml::to_string_pretty(&self)?;
        let buf = text.as_bytes();
        file.write_all(buf)?;

        Ok(self)
    }
}

/// # Errors
/// Will return `Err` if settings fails to load.
pub async fn load_settings(swarm: Swarm) -> Result<()> {
    let path = LocalSettings::path()?;
    if !path.exists() {
        tokio::fs::create_dir(&path).await?;
    }

    let mut entries = tokio::fs::read_dir(&path).await?;
    let mut usernames = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let Some(username) = file_name.strip_suffix(".toml") else {
            continue;
        };

        usernames.push(str!(username));
    }

    if usernames.is_empty() {
        let number = fastrand::u8(u8::MIN..u8::MAX);
        let username = format!("ExampleBot{number}");
        usernames.push(username);
    }

    for username in usernames {
        let settings = LocalSettings::new(&username)?.load()?.save()?;
        let account = match settings.auth_mode {
            AuthMode::Offline => Account::offline(&username),
            AuthMode::Online => Account::microsoft(&username).await?,
        };

        tokio::time::sleep(Duration::from_secs(5)).await;
        let client = if let Some(server_address) = settings.server_address.clone() {
            let Ok(resolved_address) = resolve::resolve_address(&server_address).await else {
                bail!("Failed to resolve server address")
            };

            let opts = JoinOpts::new()
                .custom_server_addr(server_address)
                .custom_socket_addr(resolved_address);

            swarm.add_with_opts(&account, NoState, &opts).await
        } else {
            swarm.add(&account, NoState).await /* Use the default server address */
        };

        let mut world = client.ecs.write();
        world.commands().entity(client.entity).insert(settings);
    }

    Ok(())
}
