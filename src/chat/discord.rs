use std::collections::VecDeque;

use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
};
use bevy_discord::{bot::events::BMessage, http::DiscordHttpResource, DiscordSet};
use serenity::json::json;

use crate::prelude::*;

pub struct DiscordChatPlugin;

impl Plugin for DiscordChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::handle_message_events.after(DiscordSet),
                Self::handle_whisper_events.before(DiscordSet),
            ),
        );
    }
}

impl DiscordChatPlugin {
    pub fn handle_message_events(
        mut message_events: EventReader<BMessage>,
        mut command_events: EventWriter<CommandEvent>,
        mut query: Query<Entity, (With<Player>, With<LocalEntity>)>,
        settings: Res<GlobalSettings>,
    ) {
        for event in message_events.read() {
            let mut events = Vec::new();
            for entity in &mut query {
                let message = event.new_message.clone();
                let mut args = message.content.split(' ').collect::<VecDeque<_>>();
                let Some(alias) = args.pop_front() else {
                    continue; /* Command Missing */
                };

                let Some(command) = ChatCmds::find(&alias.replace(&settings.command_prefix, ""))
                else {
                    continue; /* Command Invalid */
                };

                if settings.whitelist
                    && !settings
                        .whitelisted
                        .iter()
                        .filter_map(|(uuid, user_id)| {
                            user_id.as_ref().map(|user_id| (*uuid, user_id))
                        })
                        .any(|(_, user_id)| user_id == &str!(message.author.id))
                {
                    let http = event.ctx.http.clone();
                    let prefix = settings.command_prefix.clone();
                    let user_id = str!(message.author.id);
                    tokio::spawn(async move {
                        let content = [
                            str!("Your Discord and Minecraft accounts are not currently linked."),
                            format!("To link via in-game, message the bot the following command: `{prefix}whitelist link {user_id}`"),
                            format!("To link via Discord, run the following command with your `auth.aristois.net` code `{prefix}whitelist link <code>`"),
                        ];

                        let map = json!({
                            "content": content.join("\n"),
                        });

                        if let Err(error) =
                            http.send_message(message.channel_id, vec![], &map).await
                        {
                            error!("{error}");
                        };
                    });

                    continue;
                }

                events.push(CommandEvent {
                    entity,
                    args: args.into_iter().map(String::from).collect(),
                    command,
                    source: CommandSource::Discord(message.channel_id),
                    sender: CommandSender::Discord(message.author.id),
                });
            }

            command_events.send_batch(events);
        }
    }

    pub fn handle_whisper_events(
        mut whisper_events: EventReader<WhisperEvent>,
        discord: Res<DiscordHttpResource>,
    ) {
        for event in whisper_events.read() {
            if let CommandSource::Discord(channel_id) = event.source {
                let content = event.content.clone();
                let client = discord.client();

                tokio::spawn(async move {
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
}
