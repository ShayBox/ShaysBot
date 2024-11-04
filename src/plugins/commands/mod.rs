pub mod prelude;

mod discord;
mod pearl;
mod playtime;
mod seen;

use std::collections::{HashMap, VecDeque};

use azalea::{
    app::{App, Plugin, Update},
    chat::{handle_send_chat_event, ChatPacketKind, ChatReceivedEvent, SendChatKindEvent},
    ecs::prelude::*,
    prelude::*,
};
use ncr::AesKey;
use serenity::all::ChannelId;

use crate::{
    ncr::{find_encryption, try_encrypt, EncryptionType, KEY},
    settings::Settings,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Pearl,
    Playtime,
    Seen,
}

#[derive(Clone, Debug)]
pub enum CommandSource {
    Discord(ChannelId),
    Minecraft(Option<EncryptionType>),
}

#[derive(Debug, Event)]
pub struct CommandEvent {
    pub source:  CommandSource,
    pub entity:  Entity,
    pub sender:  String,
    pub command: Command,
    pub args:    VecDeque<String>,
}

#[derive(Debug, Event)]
pub struct WhisperEvent {
    pub entity:  Entity,
    pub source:  CommandSource,
    pub sender:  String,
    pub content: String,
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

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandEvent>()
            .add_event::<WhisperEvent>()
            .insert_resource(Registry::default())
            .add_systems(
                Update,
                (
                    handle_chat_received_event,
                    handle_minecraft_whisper_event.before(handle_send_chat_event),
                )
                    .chain(),
            );
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_chat_received_event(
    mut events: EventReader<ChatReceivedEvent>,
    mut command_events: EventWriter<CommandEvent>,
    registry: Res<Registry>,
    settings: Res<Settings>,
) {
    for event in events.read() {
        let (sender, content) = event.packet.split_sender_and_content();
        let (sender, content) = if let Some(sender) = sender {
            (sender, content)
        } else if let Some((_whole, sender, content)) = regex_captures!(
            r"^(?:\[.+\] )?([a-zA-Z_0-9]{1,16}) (?:> )?(?:whispers: |-> )?(.+)$",
            &content
        ) {
            (sender.to_string(), content.to_string())
        } else {
            continue;
        };

        let key = AesKey::decode_base64(&settings.encryption.key).unwrap_or_else(|_| KEY.clone());
        let (encryption, content) = find_encryption(&content, &key);
        let Some((args, command)) = registry.find_command(&content, &settings.chat_prefix) else {
            continue;
        };

        command_events.send(CommandEvent {
            source: CommandSource::Minecraft(encryption),
            entity: event.entity,
            sender,
            command: *command,
            args,
        });
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_minecraft_whisper_event(
    mut chat_kind_events: EventWriter<SendChatKindEvent>,
    mut whisper_events: EventReader<WhisperEvent>,
    settings: Res<Settings>,
) {
    for event in whisper_events.read() {
        if let CommandSource::Minecraft(encryption) = event.source {
            if settings.quiet {
                continue;
            }

            let content = try_encrypt(&settings.encryption, encryption, event.content.clone());

            chat_kind_events.send(SendChatKindEvent {
                entity:  event.entity,
                kind:    ChatPacketKind::Command,
                content: format!("w {} {content}", event.sender),
            });
        }
    }
}
