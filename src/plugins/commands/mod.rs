pub mod handlers;
pub mod prelude;

mod pearl;
mod playtime;
mod seen;
mod whitelist;

use std::collections::{HashMap, VecDeque};

use azalea::{ecs::prelude::*, prelude::*};
use serenity::all::{ChannelId, UserId};

use crate::ncr::EncryptionType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Pearl,
    Playtime,
    Seen,
    Whitelist,
}

#[derive(Clone, Debug)]
pub enum CommandSender {
    Discord(UserId),
    Minecraft(String),
}

#[derive(Clone, Debug)]
pub enum CommandSource {
    Discord(ChannelId),
    Minecraft(Option<EncryptionType>),
}

#[derive(Clone, Debug, Event)]
pub struct CommandEvent {
    pub entity:  Entity,
    pub args:    VecDeque<String>,
    pub command: Command,
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

#[derive(Default, Resource)]
pub struct Registry(HashMap<String, Command>);

impl Registry {
    pub fn register(&mut self, alias: &str, command: Command) {
        self.0.insert(alias.into(), command);
    }

    pub fn find_command(
        &self,
        content: &str,
        prefix: &str,
    ) -> Option<(VecDeque<String>, &Command)> {
        let mut args = content
            .split(' ')
            .map(String::from)
            .collect::<VecDeque<_>>();

        let alias = args.pop_front()?;
        let (_, command) = self
            .0
            .iter()
            .find(|cmd| format!("{}{}", prefix, cmd.0) == alias)?;

        Some((args, command))
    }
}
