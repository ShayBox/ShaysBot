use std::collections::VecDeque;

use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
};
use bevy_discord::{events::bot::BMessage, res::DiscordHttpResource, DiscordSet};
use serenity::json::json;

use crate::prelude::*;

/// Discord chat command parsing integration
pub struct DiscordParserPlugin;

impl Plugin for DiscordParserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::handle_message_events.after(DiscordSet),
                Self::handle_send_msg_events.before(DiscordSet),
            ),
        );
    }
}

impl DiscordParserPlugin {
    pub fn handle_message_events(
        mut message_events: EventReader<BMessage>,
        mut cmd_events: EventWriter<CmdEvent>,
        query: Query<Entity, (With<Player>, With<LocalEntity>)>,
        settings: Res<GlobalSettings>,
    ) {
        for event in message_events.read() {
            let message = event.new_message.clone();
            let mut args = message
                .content
                .split(' ')
                .map(String::from)
                .collect::<VecDeque<_>>();
            let Some(alias) = args.pop_front() else {
                continue; /* Command Missing */
            };

            let Some(cmd) = Cmds::find(&alias.replace(&settings.command_prefix, "")) else {
                continue; /* Command Invalid */
            };

            if settings.whitelist_only
                && !settings
                    .users
                    .iter()
                    .any(|(_, user)| user.discord_id == message.author.id)
            {
                let is_whitelist_link = matches!(
                    (cmd, args.front().map(String::as_str)),
                    (Cmds::Whitelist(_), Some("link"))
                );

                if !is_whitelist_link {
                    let http = event.ctx.http.clone();
                    let prefix = settings.command_prefix.clone();
                    let user_id = str!(message.author.id);

                    tokio::spawn(async move {
                        let content = format!(
                            "Your Discord and Minecraft accounts are not currently linked.\n\
                            To link via in-game, message the bot the following command: `{prefix}whitelist link {user_id}`\n\
                            To link via Discord, run the following command with your `auth.aristois.net` code `{prefix}whitelist link <code>`"
                        );

                        if let Err(error) = http
                            .send_message(
                                message.channel_id,
                                vec![],
                                &json!({ "content": content }),
                            )
                            .await
                        {
                            error!("{error}");
                        }
                    });

                    continue;
                }
            }

            let mut cmd_event = CmdEvent {
                args: args.clone(),
                cmd,
                entity: None,
                message: false,
                source: CmdSource::Discord(message.channel_id),
                sender: CmdSender::Discord(message.author.id),
            };

            cmd_events.send_batch(std::iter::once(cmd_event.clone()).chain(query.iter().map(
                |entity| {
                    cmd_event.entity = Some(entity);
                    cmd_event.clone()
                },
            )));
        }
    }

    pub fn handle_send_msg_events(
        mut msg_events: EventReader<MsgEvent>,
        discord: Option<Res<DiscordHttpResource>>,
    ) {
        let Some(discord) = discord else {
            return;
        };

        for event in msg_events.read() {
            let client = discord.client();
            let content = event.content.clone();
            let CmdSource::Discord(channel_id) = event.source else {
                continue;
            };

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
