#[cfg(feature = "bot")]
pub mod discord;
#[cfg(feature = "api")]
pub mod http_api;
pub mod minecraft;

#[cfg(feature = "api")]
use std::sync::Mutex;
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use azalea::{ecs::prelude::*, prelude::*};
#[cfg(feature = "bot")]
use serenity::all::{ChannelId, UserId};
use strum::IntoEnumIterator;
#[cfg(feature = "api")]
use tiny_http::Request;
use uuid::Uuid;

use crate::prelude::*;

pub trait Cmd {
    fn aliases(&self) -> Vec<&'static str>;
}

/// Compile time checked list of commands
#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum Cmds {
    Join(JoinCommandPlugin),
    Leave(LeaveCommandPlugin),
    Pearl(PearlCommandPlugin),
    Playtime(PlaytimeCommandPlugin),
    Seen(SeenCommandPlugin),
    Whitelist(WhitelistCommandPlugin),
}

impl Cmds {
    #[must_use]
    pub fn find(alias: &str) -> Option<Self> {
        Self::iter().find(|cmds| match cmds {
            Self::Join(cmd) => cmd.aliases().contains(&alias),
            Self::Leave(cmd) => cmd.aliases().contains(&alias),
            Self::Pearl(cmd) => cmd.aliases().contains(&alias),
            Self::Playtime(cmd) => cmd.aliases().contains(&alias),
            Self::Seen(cmd) => cmd.aliases().contains(&alias),
            Self::Whitelist(cmd) => cmd.aliases().contains(&alias),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CmdSender {
    #[cfg(feature = "api")]
    ApiServer(Uuid),
    #[cfg(feature = "bot")]
    Discord(UserId),
    Minecraft(Uuid),
}

#[derive(Clone, Debug)]
pub enum CmdSource {
    #[cfg(feature = "api")]
    ApiServer(Arc<Mutex<Option<Request>>>),
    #[cfg(feature = "bot")]
    Discord(ChannelId),
    Minecraft(Option<EncryptionType>),
}

#[derive(Clone, Debug, Event)]
pub struct CmdEvent {
    /// Optional command arguments
    pub args:    VecDeque<String>,
    pub cmd:     Cmds,
    pub entity:  Option<Entity>,
    pub message: bool,
    pub sender:  CmdSender,
    pub source:  CmdSource,
}

#[derive(Clone, Debug, Event)]
pub struct MsgEvent {
    pub content: String,
    pub entity:  Option<Entity>,
    pub sender:  CmdSender,
    pub source:  CmdSource,
    pub status:  u16,
}

#[derive(Default, Resource)]
pub struct CmdCooldown(HashMap<String, Instant>);

impl CmdCooldown {
    fn check(&mut self, sender: &str, duration: Duration) -> bool {
        if let Some(instant) = self.0.get(sender) {
            if instant.elapsed() < duration {
                return true; /* Cooldown */
            }
        } else {
            self.0.insert(str!(sender), Instant::now());
        }

        false
    }
}
