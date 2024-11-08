use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use azalea::{
    app::{App, Plugin, Update},
    chat::{handle_send_chat_event, ChatPacketKind, ChatReceivedEvent, SendChatKindEvent},
    ecs::prelude::*,
    prelude::*,
    TabList,
};
use ncr::AesKey;

use crate::{
    ncr::{find_encryption, try_encrypt, KEY},
    plugins::commands::{CommandEvent, CommandSender, CommandSource, Registry, WhisperEvent},
    settings::Settings,
};

#[derive(Resource, Default)]
pub struct CommandCooldowns {
    cooldowns: HashMap<String, Instant>,
}

impl CommandCooldowns {
    fn check_cooldown(&self, player: &str, duration: Duration) -> bool {
        if let Some(last_use) = self.cooldowns.get(player) {
            if last_use.elapsed() < duration {
                return false;
            }
        }
        true
    }

    fn update_cooldown(&mut self, player: String) {
        self.cooldowns.insert(player, Instant::now());
    }

    fn cleanup_expired(&mut self, duration: Duration) {
        self.cooldowns.retain(|_, time| time.elapsed() < duration);
    }
}

pub struct MinecraftCommandsPlugin;

impl Plugin for MinecraftCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandEvent>()
            .add_event::<WhisperEvent>()
            .insert_resource(Registry::default())
            .insert_resource(CommandCooldowns::default())
            .add_systems(
                Update,
                (
                    handle_chat_received_event,
                    handle_minecraft_whisper_event.before(handle_send_chat_event),
                    cleanup_cooldowns,
                )
                    .chain(),
            );
    }
}

fn cleanup_cooldowns(mut cooldowns: ResMut<CommandCooldowns>, settings: Res<Settings>) {
    let duration = Duration::from_secs(settings.command_cooldown);
    cooldowns.cleanup_expired(duration);
}

pub fn handle_chat_received_event(
    mut events: EventReader<ChatReceivedEvent>,
    mut command_events: EventWriter<CommandEvent>,
    mut cooldowns: ResMut<CommandCooldowns>,
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

        // Check cooldown
        let duration = Duration::from_secs(settings.command_cooldown);
        if !cooldowns.check_cooldown(&sender, duration) {
            continue;
        }

        // Update cooldown
        cooldowns.update_cooldown(sender.clone());

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
