use std::collections::VecDeque;

use azalea::{
    app::{App, Plugin, Update},
    chat::{handle_send_chat_event, ChatKind, ChatReceivedEvent, SendChatKindEvent},
    ecs::prelude::*,
    TabList,
};
use ncr::AesKey;

use crate::{
    commands::{
        handlers::Cooldown,
        CommandEvent,
        CommandSender,
        CommandSource,
        Commands,
        WhisperEvent,
    },
    encryption::{find_encryption, try_encrypt, KEY},
    settings::{BotSettings, Settings},
};

pub struct MinecraftCommandsPlugin;

impl Plugin for MinecraftCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cooldown::default())
            .add_event::<CommandEvent>()
            .add_event::<WhisperEvent>()
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
    mut chat_received_events: EventReader<ChatReceivedEvent>,
    mut command_events: EventWriter<CommandEvent>,
    mut cooldown: ResMut<Cooldown>,
    query: Query<&TabList>,
    settings: Res<Settings>,
) {
    let mut events = Vec::new();
    for event in chat_received_events.read() {
        let (username, content) = event.packet.split_sender_and_content();
        let (username, content) = if let Some(username) = username {
            (username, content) /* Vanilla Server Format */
        } else if let Some((_whole, username, content)) = regex_captures!(
            r"^(?:\[.+\] )?([a-zA-Z_0-9]{1,16}) (?:> )?(?:whispers: |-> )?(.+)$",
            &content /* Custom Server Formats */
        ) {
            (username.to_string(), content.to_string())
        } else {
            continue;
        };

        let Ok(tab_list) = query.get(event.entity) else {
            return; /* Not Connected */
        };

        let Some((uuid, _)) = tab_list.iter().find(|(_, i)| i.profile.name == username) else {
            continue; /* Not Online */
        };

        if settings.whitelist && !settings.whitelisted.contains_key(uuid) {
            continue; /* Not Whitelisted */
        }

        let key = AesKey::decode_base64(&settings.encryption.key).unwrap_or_else(|_| KEY.clone());
        let (encryption, content) = find_encryption(&content, &key);

        let mut args = content.split(' ').collect::<VecDeque<_>>();
        let Some(alias) = args.pop_front() else {
            continue; /* Command Missing */
        };

        let Some(command) = Commands::find(&alias.replace(&settings.command_prefix, "")) else {
            continue; /* Command Invalid */
        };

        if cooldown.check(&username, settings.command_cooldown) {
            info!("Command on cooldown");
            continue; /* Command Cooldown */
        }

        events.push(CommandEvent {
            entity: event.entity,
            args: args.into_iter().map(String::from).collect(),
            command,
            sender: CommandSender::Minecraft(*uuid),
            source: CommandSource::Minecraft(encryption),
        });
    }

    command_events.send_batch(events);
}

pub fn handle_minecraft_whisper_event(
    mut chat_kind_events: EventWriter<SendChatKindEvent>,
    mut whisper_events: EventReader<WhisperEvent>,
    query: Query<(&TabList, &BotSettings)>,
    settings: Res<Settings>,
) {
    for mut event in whisper_events.read().cloned() {
        #[rustfmt::skip]
        let (
            CommandSource::Minecraft(type_encryption),
            CommandSender::Minecraft(uuid)
        ) = (event.source, event.sender) else {
            continue;
        };

        let Ok((tab_list, bot_settings)) = query.get(event.entity) else {
            return;
        };

        if bot_settings.disable_responses {
            continue; /* Responses Disabled */
        }

        let Some(username) = tab_list
            .iter()
            .find(|(_, info)| info.profile.uuid == uuid)
            .map(|(_, info)| info.profile.name.clone())
        else {
            continue; /* Player Offline */
        };

        try_encrypt(&mut event.content, &settings.encryption, type_encryption);

        chat_kind_events.send(SendChatKindEvent {
            entity:  event.entity,
            kind:    ChatKind::Command,
            content: format!("w {username} {}", event.content),
        });
    }
}
