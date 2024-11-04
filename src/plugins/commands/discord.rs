use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
};
use bevy_discord::{bot::events::BMessage, http::DiscordHttpResource, runtime::tokio_runtime};
use serenity::json::json;

use super::{CommandEvent, CommandSource, Registry, WhisperEvent};
use crate::settings::Settings;

pub struct DiscordCommandsPlugin;

impl Plugin for DiscordCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_message_event, handle_discord_whisper_event));
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
pub fn handle_discord_whisper_event(
    mut whisper_events: EventReader<WhisperEvent>,
    discord: Res<DiscordHttpResource>,
) {
    for event in whisper_events.read() {
        if let CommandSource::Discord(channel_id) = event.source {
            let content = event.content.clone();
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
    }
}
