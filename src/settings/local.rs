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
    protocol::{resolver, ServerAddress},
    swarm::Swarm,
    JoinOpts,
    NoState,
    Vec3,
};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serenity::all::ChannelId;
use smart_default::SmartDefault;

#[derive(Clone, Component, Deserialize, Serialize, SmartDefault)]
#[serde(default)]
pub struct LocalSettings {
    #[serde(skip)]
    path: PathBuf,

    /// Minecraft account authentication mode.
    pub auth_mode: AuthMode,

    /// Anti Afk module settings.
    pub anti_afk: AntiAfk,

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
    pub discord_channel: ChannelId,

    /// Minecraft account server address. (Optional)
    pub server_address: Option<ServerAddress>,
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
    #[default(false)]
    pub enabled: bool,

    #[default(20 * 60)]
    #[serde_as(as = "DisplayFromStr")]
    pub delay_ticks: u128,
}

#[derive(Clone, Deserialize, Serialize, SmartDefault)]
#[serde(rename = "auto_exit")] /* Deprecated: 0.11 */
#[serde(default)]
pub struct AutoLeave {
    /// Requires whitelist
    #[default(true)]
    pub unknown_player: bool,

    /// Requires zenith proxy
    #[default(true)]
    pub zenith_proxy: bool,
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
    /// Location name
    pub location: String,

    /// Position & Radius to idle after pearling.
    #[serde(rename = "idle")]
    pub idle_goal: IdleGoal,
}

#[serde_as]
#[derive(Clone, Default, PartialEq, Deserialize, Serialize)]
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
    /// # Panics
    /// Will panic if `std::env::current_exe` fails.
    #[must_use]
    pub fn new(username: &str) -> Self {
        let mut path = std::env::current_exe().unwrap();
        path.set_file_name(format!("local-settings/{username}"));
        path.set_extension("toml");

        Self::from(path)
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

pub struct LocalSettingsPlugin;

impl Plugin for LocalSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::handle_startup);
    }
}

impl LocalSettingsPlugin {
    pub fn handle_startup(swarm: Res<Swarm>) {
        let swarm = swarm.clone();

        tokio::spawn(async move {
            if let Err(error) = load_settings(swarm).await {
                error!("There was an error loading local settings: {error}");
                std::process::exit(1);
            }
        });
    }
}

/// # Errors
/// Will return `Err` if settings fails to load.
pub async fn load_settings(mut swarm: Swarm) -> Result<()> {
    let current_exe_path = std::env::current_exe()?;
    let current_dir_path = current_exe_path.parent().context("None")?;
    let local_settings_path = current_dir_path.join("local-settings");

    if !local_settings_path.exists() {
        tokio::fs::create_dir(&local_settings_path).await?;
    }

    let mut entries = tokio::fs::read_dir(&local_settings_path).await?;
    let mut usernames = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let Some(username) = file_name.strip_suffix(".toml") else {
            continue;
        };

        if username == "global-settings" {
            continue;
        }

        usernames.push(str!(username));
    }

    if usernames.is_empty() {
        let number = fastrand::u8(u8::MIN..u8::MAX);
        let username = format!("ExampleBot{number}");
        usernames.push(username);
    }

    for username in usernames {
        let settings = LocalSettings::new(&username).load()?.save()?;
        let account = match settings.auth_mode {
            AuthMode::Offline => Account::offline(&username),
            AuthMode::Online => Account::microsoft(&username).await?,
        };

        tokio::time::sleep(Duration::from_secs(5)).await;
        let client = if let Some(server_address) = settings.server_address.clone() {
            let Ok(resolved_address) = resolver::resolve_address(&server_address).await else {
                bail!("Failed to resolve server address")
            };
            let opts = JoinOpts::new()
                .custom_address(server_address)
                .custom_resolved_address(resolved_address);
            swarm.add_with_opts(&account, NoState, &opts).await?
        } else {
            swarm.add(&account, NoState).await? /* Use the default server address */
        };

        let mut world = client.ecs.lock();
        world.commands().entity(client.entity).insert(settings);
    }

    Ok(())
}
