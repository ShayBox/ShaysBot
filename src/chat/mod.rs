pub mod discord;
pub mod minecraft;

use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use azalea::{ecs::prelude::*, prelude::*};
use serenity::all::{ChannelId, UserId};
use uuid::Uuid;

use crate::prelude::*;

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
    pub command: ChatCmds,
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
pub struct CommandCooldown(HashMap<String, Instant>);

impl CommandCooldown {
    fn check(&mut self, sender: &str, duration: Duration) -> bool {
        if let Some(instant) = self.0.get(sender) {
            if instant.elapsed() < duration {
                return true;
            }
        } else {
            self.0.insert(str!(sender), Instant::now());
        }

        false
    }
}
