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
use bevy_discord::{
    bot::{events::BMessage, DiscordBotRes},
    runtime::tokio_runtime
    ,
};
use serenity::{all::ChannelId, json::json};

use crate::settings::Settings;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    Pearl,
    Playtime,
    Seen,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommandSource {
    Discord(ChannelId),
    Minecraft,
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
    source:  CommandSource,
    entity:  Entity,
    sender:  String,
    content: String,
}

#[derive(Default, Resource)]
pub struct Registry(HashMap<String, Command>);

impl Registry {
    fn register(&mut self, alias: &str, command: Command) {
        self.0.insert(alias.into(), command);
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
        let entity = query.single_mut();
        let http = event.ctx.http.clone();
        let message = event.new_message.clone();
        let Some((args, command)) =
            try_find_command(registry.0.clone(), &message.content, &settings.chat_prefix)
        else {
            continue;
        };

        let Some(sender) = args.front().map(String::to_owned) else {
            tokio_runtime().spawn(async move {
                let map = &json!({
                    "content": "[404] Missing Player Name!"
                });

                if let Err(error) = http.send_message(message.channel_id, Vec::new(), map).await {
                    eprintln!("{error}");
                };
            });

            continue;
        };

        command_events.send(CommandEvent {
            source: CommandSource::Discord(message.channel_id),
            entity,
            sender,
            command,
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
            r"^(?:\[.+\] )?([a-zA-Z_0-9]{1,16}) (?:> )?(?:whispers: )?(.+)$",
            &content
        ) {
            (sender.to_string(), content.to_string())
        } else {
            continue;
        };

        let Some((args, command)) =
            try_find_command(registry.0.clone(), &content, &settings.chat_prefix)
        else {
            continue;
        };

        command_events.send(CommandEvent {
            source: CommandSource::Minecraft,
            entity: event.entity,
            sender,
            command,
            args,
        });
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_whisper_event(
    mut whisper_events: EventReader<WhisperEvent>,
    mut chat_kind_events: EventWriter<SendChatKindEvent>,
    discord_bot_res: Res<DiscordBotRes>,
) {
    for event in whisper_events.read() {
        let content = event.content.clone();

        match event.source {
            CommandSource::Discord(channel_id) => {
                if let Ok(http) = discord_bot_res.get_http() {
                    tokio_runtime().spawn(async move {
                        let map = &json!({
                            "content": content,
                        });

                        if let Err(error) = http.send_message(channel_id, Vec::new(), map).await {
                            eprintln!("{error}");
                        }
                    });
                }
            }
            CommandSource::Minecraft => {
                chat_kind_events.send(SendChatKindEvent {
                    entity:  event.entity,
                    kind:    ChatPacketKind::Command,
                    content: format!("w {} {}", event.sender, event.content),
                });
            }
        }
    }
}

pub fn try_find_command(
    commands: HashMap<String, Command>,
    content: &str,
    prefix: &str,
) -> Option<(VecDeque<String>, Command)> {
    let mut args = content
        .split(' ')
        .map(String::from)
        .collect::<VecDeque<_>>();

    let alias = args.pop_front()?;
    let (_, command) = commands
        .into_iter()
        .find(|cmd| format!("{}{}", prefix, cmd.0) == alias)?;

    Some((args, command))
}
