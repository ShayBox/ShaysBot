use std::collections::VecDeque;

use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
};
use bevy_discord::{bot::events::BMessage, http::DiscordHttpResource, runtime::tokio_runtime};
use serenity::json::json;

use crate::{
    commands::{CommandEvent, CommandSender, CommandSource, Commands, WhisperEvent},
    settings::Settings,
};

pub struct DiscordCommandsPlugin;

impl Plugin for DiscordCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_message_event, handle_discord_whisper_event));
    }
}

pub fn handle_message_event(
    mut command_events: EventWriter<CommandEvent>,
    mut message_events: EventReader<BMessage>,
    mut query: Query<Entity, (With<Player>, With<LocalEntity>)>,
    settings: Res<Settings>,
) {
    for event in message_events.read() {
        let Ok(entity) = query.get_single_mut() else {
            continue;
        };

        let message = event.new_message.clone();
        let mut args = message.content.split(' ').collect::<VecDeque<_>>();
        let Some(alias) = args.pop_front() else {
            continue; /* Command Missing */
        };

        let Some(command) = Commands::find(&alias.replace(&settings.command_prefix, "")) else {
            continue; /* Command Invalid */
        };

        if !settings.whitelist.is_empty()
            && !settings
                .whitelist
                .iter()
                .filter_map(|(uuid, user_id)| user_id.as_ref().map(|user_id| (*uuid, user_id)))
                .any(|(_, user_id)| user_id == &message.author.id.to_string())
        {
            let http = event.ctx.http.clone();
            let prefix = settings.command_prefix.clone();
            let user_id = message.author.id.to_string();
            tokio_runtime().spawn(async move {
                let content = [
                    String::from("[404] Your Discord and Minecraft accounts aren't linked,"),
                    format!("In Game: `{prefix}whitelist link {user_id}`"),
                    format!("Discord: `{prefix}whitelist link (auth.aristois.net)`"),
                ];

                let map = json!({
                    "content": content.join("\n"),
                });

                if let Err(error) = http.send_message(message.channel_id, vec![], &map).await {
                    error!("{error}");
                };
            });

            continue;
        }

        command_events.send(CommandEvent {
            entity,
            args: args.into_iter().map(String::from).collect(),
            command,
            source: CommandSource::Discord(message.channel_id),
            sender: CommandSender::Discord(message.author.id),
        });
    }
}

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
