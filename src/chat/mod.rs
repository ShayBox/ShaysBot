#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "discord")]
pub mod discord;
pub mod minecraft;

#[cfg(feature = "api")]
use std::sync::Mutex;
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use azalea::{ecs::prelude::*, prelude::*};
#[cfg(feature = "discord")]
use serenity::all::{ChannelId, UserId};
#[cfg(feature = "api")]
use tiny_http::Request;
use uuid::Uuid;

use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum CommandSender {
    #[cfg(feature = "api")]
    ApiServer(Uuid),
    #[cfg(feature = "discord")]
    Discord(UserId),
    Minecraft(Uuid),
}

#[derive(Clone, Debug)]
pub enum CommandSource {
    #[cfg(feature = "api")]
    ApiServer(Arc<Mutex<Option<Request>>>),
    #[cfg(feature = "discord")]
    Discord(ChannelId),
    Minecraft(Option<EncryptionType>),
}

#[derive(Clone, Debug, Event)]
pub struct CommandEvent {
    pub entity:  Entity,
    pub args:    VecDeque<String>,
    pub command: ChatCmds,
    pub message: bool,
    pub sender:  CommandSender,
    pub source:  CommandSource,
}

#[derive(Clone, Debug, Event)]
pub struct WhisperEvent {
    pub entity:  Entity,
    pub content: String,
    pub sender:  CommandSender,
    pub source:  CommandSource,
    pub status:  u16,
}

#[derive(Default, Resource)]
pub struct CommandCooldown(HashMap<String, Instant>);

impl CommandCooldown {
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
