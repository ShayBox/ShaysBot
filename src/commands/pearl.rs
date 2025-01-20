use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::Position,
    BlockPos,
    TabList,
};

use crate::prelude::*;

/// Automatically pull pearls remotely.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PearlCommandPlugin;

impl ChatCmd for PearlCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["pearl", "tp", "teleport", "warp", "home"]
    }
}

impl Plugin for PearlCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_pearl_command_events
                .ambiguous_with_all()
                .before(AutoPearlPlugin::handle_goto_pearl_events)
                .before(MinecraftChatPlugin::handle_send_whisper_events)
                .after(MinecraftChatPlugin::handle_chat_received_events),
        );
    }
}

impl PearlCommandPlugin {
    #[allow(clippy::too_many_lines)]
    pub fn handle_pearl_command_events(
        mut command_events: EventReader<CommandEvent>,
        mut pearl_events: EventWriter<PearlGotoEvent>,
        mut whisper_events: EventWriter<WhisperEvent>,
        query: Query<(&TabList, &Position, &LocalSettings)>,
        settings: Query<&LocalSettings>,
        global_settings: Res<GlobalSettings>,
        stasis_chambers: Res<StasisChambers>,
    ) {
        for mut event in command_events.read().cloned() {
            let ChatCmds::Pearl(_plugin) = event.command else {
                continue;
            };

            let Ok((tab_list, position, local_settings)) = query.get(event.entity) else {
                continue;
            };

            let mut locations = settings
                .iter()
                .cloned()
                .map(|ls| ls.auto_pearl.location)
                .collect::<Vec<_>>();

            locations.sort();
            locations.dedup();

            let settings = settings.iter().cloned().collect::<Vec<_>>();
            let mut whisper_event = WhisperEvent {
                entity:  event.entity,
                source:  event.source.clone(),
                sender:  event.sender,
                status:  406,
                content: format!("Invalid location | Locations: {}", locations.join(", ")),
            };

            if !local_settings.auto_pearl.enabled {
                continue; /* Auto Pearl Disabled */
            }

            let uuid = match event.sender {
                #[cfg(feature = "api")]
                CommandSender::ApiServer => {
                    let Some(username) = event.args.pop_front() else {
                        whisper_event.content = str!("Missing player name");
                        whisper_event.status = 404;
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    };

                    let Some((uuid, _info)) = tab_list.iter().find(|(_, info)| {
                        info.profile.name.to_lowercase() == username.to_lowercase()
                    }) else {
                        whisper_event.content = format!("{username} is not online");
                        whisper_event.status = 404;
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    };

                    *uuid
                }
                #[cfg(feature = "discord")]
                CommandSender::Discord(user_id) => {
                    let Some(username) = event.args.pop_front() else {
                        whisper_event.content = str!("Missing player name");
                        whisper_event.status = 404;
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    };

                    let Some((uuid, _info)) = tab_list.iter().find(|(_, info)| {
                        info.profile.name.to_lowercase() == username.to_lowercase()
                    }) else {
                        whisper_event.content = format!("{username} is not online");
                        whisper_event.status = 404;
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    };

                    if global_settings.whitelist {
                        let Some(whitelist) = global_settings.whitelisted.get(uuid) else {
                            command_events.clear();
                            return; /* Not Whitelisted */
                        };

                        let Some(discord_id) = whitelist else {
                            whisper_event.content = str!("That account isn't linked to you");
                            whisper_event.status = 403;
                            whisper_events.send(whisper_event);
                            command_events.clear();
                            return;
                        };

                        if discord_id != &str!(user_id) {
                            whisper_event.content = str!("That account isn't linked to you");
                            whisper_event.status = 403;
                            whisper_events.send(whisper_event);
                            command_events.clear();
                            return;
                        }
                    }

                    *uuid
                }
                CommandSender::Minecraft(uuid) => uuid,
            };

            let local_settings = match settings.first() {
                Some(local_settings) if settings.len() == 1 => local_settings,
                _ => {
                    /* Multi-Account Swarm */
                    if let Some(location) = event.args.pop_front() {
                        if location == local_settings.auto_pearl.location {
                            local_settings
                        } else if let CommandSource::Minecraft(_) = event.source {
                            if event.message {
                                // TODO: Redirect to appropriate bot
                                local_settings /* Local Chat */
                            } else {
                                continue; /* Global Chat */
                            }
                        } else {
                            continue; /* Discord Chat */
                        }
                    } else {
                        match event.source {
                            #[cfg(feature = "api")]
                            CommandSource::ApiServer(_) => {
                                whisper_events.send(whisper_event);
                                command_events.clear();
                                return;
                            }
                            #[cfg(feature = "discord")]
                            CommandSource::Discord(_) => {
                                whisper_events.send(whisper_event);
                                command_events.clear();
                                return;
                            }
                            CommandSource::Minecraft(_) => local_settings,
                        }
                    }
                }
            };

            let client_pos = BlockPos::from(position);
            let player_chambers = stasis_chambers
                .0
                .values()
                .filter(|&chamber| chamber.owner_uuid == uuid)
                .filter(|&chamber| chamber.location == local_settings.auto_pearl.location)
                .cloned()
                .map(|chamber| {
                    let distance = 1
                        + (client_pos.x - chamber.block_pos.x).abs()
                        + (client_pos.y - chamber.block_pos.y).abs()
                        + (client_pos.z - chamber.block_pos.z).abs();

                    (chamber, distance)
                });

            let count = player_chambers.clone().count().saturating_sub(1);
            let Some((chamber, _)) = player_chambers.min_by_key(|(chamber, distance)| {
                let shared_count = stasis_chambers
                    .0
                    .values()
                    .filter(|c| c.block_pos == chamber.block_pos)
                    .filter(|c| c.owner_uuid != chamber.owner_uuid)
                    .count();

                // First compare by shared count, then by distance
                (shared_count, *distance)
            }) else {
                let location = &local_settings.auto_pearl.location;
                whisper_event.content = format!("Pearl not found at {location}");
                whisper_event.status = 404;
                whisper_events.send(whisper_event);
                command_events.clear();
                return;
            };

            whisper_event.status = 200;
            whisper_event.content = match count {
                0 => str!("I'm on my way, this was your last pearl!"),
                1 => str!("I'm on my way, you have one more pearl!"),
                c => format!("I'm on my way, you have {c} more pearls."),
            };

            whisper_events.send(whisper_event);
            pearl_events.send(PearlGotoEvent(PearlEvent {
                entity:     event.entity,
                idle_goal:  local_settings.auto_pearl.idle_goal.clone(),
                block_pos:  chamber.block_pos,
                owner_uuid: chamber.owner_uuid,
            }));

            command_events.clear();
            return;
        }
    }
}
