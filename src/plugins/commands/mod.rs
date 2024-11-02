pub mod prelude;

mod pearl;
mod playtime;
mod seen;

use std::collections::{HashMap, VecDeque};

use azalea::{
    app::{App, Plugin, Update},
    chat::{handle_send_chat_event, ChatPacketKind, ChatReceivedEvent, SendChatKindEvent},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
};
use bevy_discord::{bot::events::BMessage, http::DiscordHttpResource, runtime::tokio_runtime};
use ncr::AesKey;
use serenity::{all::ChannelId, json::json};

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
    source:  CommandSource,
    entity:  Entity,
    sender:  String,
    command: Command,
    args:    VecDeque<String>,
}

#[derive(Debug, Event)]
pub struct WhisperEvent {
    entity:  Entity,
    source:  CommandSource,
    sender:  String,
    content: String,
}

#[derive(Default, Resource)]
pub struct Registry(HashMap<String, Command>);

impl Registry {
    fn register(&mut self, alias: &str, command: Command) {
        self.0.insert(alias.into(), command);
    }

    fn find_command(&self, content: &str, prefix: &str) -> Option<(VecDeque<String>, &Command)> {
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
        app.add_event::<BMessage>()
            .add_event::<CommandEvent>()
            .add_event::<WhisperEvent>()
            .insert_resource(Registry::default())
            .add_systems(
                Update,
                (
                    handle_message_event,
                    handle_chat_received_event,
                    handle_whisper_event.before(handle_send_chat_event),
                )
                    .chain(),
            );
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_message_event(
    mut command_events: EventWriter<CommandEvent>,
    mut message_events: EventReader<BMessage>,
    mut query: Query<Entity, (With<Player>, With<LocalEntity>)>,
    registry: Res<Registry>,
    settings: Res<Settings>,
) {
    for event in message_events.read() {
        let Ok(entity) = query.get_single_mut() else {
            continue;
        };

        let http = event.ctx.http.clone();
        let message = event.new_message.clone();
        let Some((args, command)) = registry.find_command(&message.content, &settings.chat_prefix)
        else {
            continue;
        };

        let Some(sender) = args.front().map(String::to_owned) else {
            tokio_runtime().spawn(async move {
                let map = &json!({
                    "content": "[404] Missing Player Name!"
                });

                if let Err(error) = http.send_message(message.channel_id, Vec::new(), map).await {
                    error!("{error}");
                };
            });

            continue;
        };

        command_events.send(CommandEvent {
            source: CommandSource::Discord(message.channel_id),
            entity,
            sender,
            command: *command,
            args,
        });
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
pub fn handle_whisper_event(
    mut chat_kind_events: EventWriter<SendChatKindEvent>,
    mut whisper_events: EventReader<WhisperEvent>,
    discord: Res<DiscordHttpResource>,
    settings: Res<Settings>,
) {
    for event in whisper_events.read() {
        let content = event.content.clone();

        match event.source {
            CommandSource::Discord(channel_id) => {
                let client = discord.client();
                tokio_runtime().spawn(async move {
                    let map = &json!({
                        "content": content,
                    });

                    if let Err(error) = client.send_message(channel_id, Vec::new(), map).await {
                        error!("{error}");
                    }
                });
            }
            CommandSource::Minecraft(encryption) => {
                if settings.quiet {
                    continue;
                }

                let content = try_encrypt(&settings.encryption, encryption, content);

                chat_kind_events.send(SendChatKindEvent {
                    entity:  event.entity,
                    kind:    ChatPacketKind::Command,
                    content: format!("w {} {content}", event.sender),
                });
            }
        }
    }
}
