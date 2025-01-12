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
                .before(DiscordChatPlugin::handle_send_whisper_events)
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
                source:  event.source,
                sender:  event.sender,
                content: format!(
                    "[404] Invalid location, Available locations: {}",
                    locations.join(", ")
                ),
            };

            let uuid = match event.sender {
                CommandSender::Minecraft(uuid) => uuid,
                CommandSender::Discord(user_id) => {
                    let Some(username) = event.args.pop_front() else {
                        whisper_event.content = str!("[404] Missing username");
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    };

                    let Some((uuid, _info)) = tab_list.iter().find(|(_, info)| {
                        info.profile.name.to_lowercase() == username.to_lowercase()
                    }) else {
                        whisper_event.content = format!("[404] {username} is not online");
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
                            whisper_event.content = str!("[404] That account isn't linked to you");
                            whisper_events.send(whisper_event);
                            return;
                        };

                        if discord_id != &str!(user_id) {
                            whisper_event.content = str!("[403] That account isn't linked to you");
                            whisper_events.send(whisper_event);
                            command_events.clear();
                            return;
                        }
                    }

                    *uuid
                }
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
                                local_settings
                            } else {
                                whisper_event.content = format!("[500] I'm not at {location}");
                                whisper_events.send(whisper_event.clone());
                                continue; /* Global Chat */
                            }
                        } else {
                            continue; /* Discord Chat */
                        }
                    } else {
                        match event.source {
                            CommandSource::Minecraft(_) => local_settings,
                            CommandSource::Discord(_) => {
                                whisper_events.send(whisper_event);
                                command_events.clear();
                                return;
                            }
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
                whisper_event.content = format!(
                    "[404] Pearl not found at {}",
                    local_settings.auto_pearl.location
                );
                whisper_events.send(whisper_event);
                command_events.clear();
                return;
            };

            whisper_event.content = match count {
                0 => str!("[200] I'm on my way, this was your last pearl!"),
                1 => str!("[200] I'm on my way, you have one more pearl!"),
                c => format!("[200] I'm on my way, you have {c} more pearls."),
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
