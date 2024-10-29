pub mod prelude;

mod pearl;

use std::collections::{HashMap, VecDeque};

use azalea::{
    app::{App, Plugin, Update},
    chat::{handle_send_chat_event, ChatPacketKind, ChatReceivedEvent, SendChatKindEvent},
    ecs::prelude::*,
    prelude::*,
};

use crate::{minecraft::prelude::*, settings::SettingsLock};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    Pearl,
}

#[derive(Debug, Event)]
pub struct CommandEvent {
    entity:  Entity,
    sender:  String,
    command: Command,
}

#[derive(Debug, Event)]
pub struct WhisperEvent {
    entity:  Entity,
    sender:  String,
    content: String,
}

#[derive(Default, Resource)]
pub struct Commands(pub HashMap<String, Command>);

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandEvent>()
            .add_event::<WhisperEvent>()
            .insert_resource(Commands::default())
            .add_systems(
                Update,
                (
                    handle_chat_received_event.before(handle_command_event),
                    handle_whisper_event.before(handle_send_chat_event),
                ),
            );
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_chat_received_event(
    mut events: EventReader<ChatReceivedEvent>,
    mut command_events: EventWriter<CommandEvent>,
    query: Query<&SettingsLock>,
    commands: Res<Commands>,
) {
    for event in events.read() {
        println!("{}", event.packet.message().to_ansi());

        let Ok(settings) = query.get(event.entity) else {
            continue;
        };

        let (sender, content) = event.packet.split_sender_and_content();
        let (sender, content) = if let Some(sender) = sender {
            (sender, content)
        } else if let Some((_whole, sender, content)) = regex_captures!(
            r"^(?:\[.+\])? ([a-zA-Z_0-9]{1,16}) (?:> )?(?:whispers: )?(.+)$",
            &content
        ) {
            (sender.to_string(), content.to_string())
        } else {
            continue;
        };

        let mut args = content.split(' ').collect::<VecDeque<_>>();
        let Some(alias) = args.pop_front() else {
            continue;
        };

        let Some((_, command)) = commands
            .0
            .clone()
            .into_iter()
            .find(|cmd| format!("{}{}", settings.0.read().chat_prefix, cmd.0) == alias)
        else {
            continue;
        };

        command_events.send(CommandEvent {
            entity: event.entity,
            sender,
            command,
        });
    }
}

pub fn handle_whisper_event(
    mut events: EventReader<WhisperEvent>,
    mut chat_kind_events: EventWriter<SendChatKindEvent>,
) {
    for event in events.read() {
        chat_kind_events.send(SendChatKindEvent {
            entity:  event.entity,
            kind:    ChatPacketKind::Command,
            content: format!("w {} {}", event.sender, event.content),
        });
    }
}
