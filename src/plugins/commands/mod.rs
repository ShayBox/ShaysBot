pub mod prelude;

mod discord;
mod pearl;
mod playtime;
mod seen;
mod whitelist;

use std::collections::{HashMap, VecDeque};

use azalea::{
    app::{App, Plugin, Update},
    chat::{handle_send_chat_event, ChatPacketKind, ChatReceivedEvent, SendChatKindEvent},
    ecs::prelude::*,
    prelude::*,
    TabList,
};
use ncr::AesKey;
use serenity::all::{ChannelId, UserId};

use crate::{
    ncr::{find_encryption, try_encrypt, EncryptionType, KEY},
    settings::Settings,
};

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

pub fn handle_chat_received_event(
    mut events: EventReader<ChatReceivedEvent>,
    mut command_events: EventWriter<CommandEvent>,
    query: Query<&TabList>,
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
        let Some((args, command)) = registry.find_command(&content, &settings.command_prefix)
        else {
            continue;
        };

        if !settings.whitelist.is_empty() {
            let Ok(tab_list) = query.get_single() else {
                continue;
            };

            let Some((uuid, _info)) = tab_list
                .iter()
                .find(|(_, info)| info.profile.name == sender)
            else {
                continue; /* Not Online */
            };

            if !settings.whitelist.contains_key(uuid) {
                continue; /* Not Whitelisted */
            }
        }

        command_events.send(CommandEvent {
            entity: event.entity,
            args,
            command: *command,
            sender: CommandSender::Minecraft(sender),
            source: CommandSource::Minecraft(encryption),
        });
    }
}

pub fn handle_minecraft_whisper_event(
    mut chat_kind_events: EventWriter<SendChatKindEvent>,
    mut whisper_events: EventReader<WhisperEvent>,
    settings: Res<Settings>,
) {
    for event in whisper_events.read().cloned() {
        let CommandSender::Minecraft(sender) = event.sender else {
            continue;
        };

        let CommandSource::Minecraft(type_encryption) = event.source else {
            continue;
        };

        if settings.disable_responses {
            continue;
        }

        let content = try_encrypt(&settings.encryption, type_encryption, event.content);

        chat_kind_events.send(SendChatKindEvent {
            entity:  event.entity,
            kind:    ChatPacketKind::Command,
            content: format!("w {sender} {content}"),
        });
    }
}
