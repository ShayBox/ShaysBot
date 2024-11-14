pub mod handlers;
pub mod prelude;

mod pearl;
mod playtime;
mod seen;
mod whitelist;

use std::collections::VecDeque;

use azalea::{ecs::prelude::*, prelude::*};
use serenity::all::{ChannelId, UserId};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::{commands::prelude::*, encryption::EncryptionType};

pub trait Command {
    fn aliases(&self) -> Vec<&'static str>;
}

/// Compile time checked list of commands
#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum Commands {
    Pearl(PearlCommandPlugin),
    Playtime(PlaytimeCommandPlugin),
    Seen(SeenCommandPlugin),
    Whitelist(WhitelistCommandPlugin),
}

impl Commands {
    fn find(alias: &str) -> Option<Self> {
        Self::iter().find(|cmds| match cmds {
            Self::Pearl(cmd) => cmd.aliases().contains(&alias),
            Self::Playtime(cmd) => cmd.aliases().contains(&alias),
            Self::Seen(cmd) => cmd.aliases().contains(&alias),
            Self::Whitelist(cmd) => cmd.aliases().contains(&alias),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CommandSender {
    Discord(UserId),
    Minecraft(Uuid),
}

#[derive(Clone, Copy, Debug)]
pub enum CommandSource {
    Discord(ChannelId),
    Minecraft(Option<EncryptionType>),
}

#[derive(Clone, Debug, Event)]
pub struct CommandEvent {
    pub entity:  Entity,
    pub args:    VecDeque<String>,
    pub command: Commands,
    pub sender:  CommandSender,
    pub source:  CommandSource,
}

#[derive(Clone, Debug, Event)]
pub struct WhisperEvent {
    pub entity:  Entity,
    pub content: String,
    pub sender:  CommandSender,
    pub source:  CommandSource,
}
