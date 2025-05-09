use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::Position,
    BlockPos,
    TabList,
};

use crate::prelude::*;

/// Automatically pull the closest stasis chamber at a `location`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PearlCommandPlugin;

impl Cmd for PearlCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["pearl", "tp", "teleport", "warp", "home"]
    }
}

impl Plugin for PearlCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_pearl_cmd_events
                .ambiguous_with_all()
                .before(AutoPearlPlugin::handle_goto_pearl_events)
                .before(MinecraftParserPlugin::handle_send_msg_events)
                .after(MinecraftParserPlugin::handle_chat_received_events),
        );
    }
}

impl PearlCommandPlugin {
    #[allow(clippy::too_many_lines)]
    #[cfg_attr(not(feature = "bot"), allow(unused_variables))]
    pub fn handle_pearl_cmd_events(
        mut cmd_events: EventReader<CmdEvent>,
        mut msg_events: EventWriter<MsgEvent>,
        mut pearl_events: EventWriter<PearlGotoEvent>,
        query: Query<(&TabList, &Position, &LocalSettings)>,
        settings: Query<&LocalSettings>,
        global_settings: Res<GlobalSettings>,
        stasis_chambers: Res<StasisChambers>,
    ) {
        for mut event in cmd_events.read().cloned() {
            let (Cmds::Pearl(_plugin), Some(entity)) = (event.cmd, event.entity) else {
                continue;
            };

            let Ok((tab_list, position, local_settings)) = query.get(entity) else {
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
            let mut msg_event = MsgEvent {
                entity:  event.entity,
                source:  event.source.clone(),
                sender:  event.sender,
                status:  406,
                content: format!("Invalid location | Locations: {}", locations.join(", ")),
            };

            if !local_settings.auto_pearl.enabled {
                continue; /* Auto Pearl Disabled */
            }

            #[allow(clippy::infallible_destructuring_match)]
            let uuid = match event.sender {
                #[cfg(feature = "api")]
                CmdSender::ApiServer(uuid) => uuid,
                #[cfg(feature = "bot")]
                CmdSender::Discord(user_id) => {
                    let Some(username) = event.args.pop_front() else {
                        msg_event.content = str!("Missing player name");
                        msg_event.status = 404;
                        msg_events.write(msg_event);
                        cmd_events.clear();
                        return;
                    };

                    let Some((uuid, _info)) = tab_list.iter().find(|(_, info)| {
                        info.profile.name.to_lowercase() == username.to_lowercase()
                    }) else {
                        msg_event.content = format!("{username} is not online");
                        msg_event.status = 404;
                        msg_events.write(msg_event);
                        cmd_events.clear();
                        return;
                    };

                    if global_settings.whitelist_only {
                        let Some(user) = global_settings.users.get(uuid) else {
                            cmd_events.clear();
                            return; /* Not Whitelisted */
                        };

                        /* Hacky way to allow any Discord account to pearl a Minecraft account. */
                        if ![str!(user_id), str!("*")].contains(&user.discord_id.to_string()) {
                            msg_event.content = str!("That account isn't linked to you");
                            msg_event.status = 403;
                            msg_events.write(msg_event);
                            cmd_events.clear();
                            return;
                        }
                    }

                    *uuid
                }
                #[allow(irrefutable_let_patterns)]
                CmdSender::Minecraft(uuid) => uuid,
            };

            let local_settings = match settings.first() {
                Some(local_settings) if settings.len() == 1 => local_settings,
                _ => {
                    /* Multi-Account Swarm */
                    if let Some(location) = event.args.pop_front() {
                        #[allow(irrefutable_let_patterns)]
                        if location == local_settings.auto_pearl.location {
                            local_settings
                        } else if let CmdSource::Minecraft(_) = event.source {
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
                            CmdSource::ApiServer(_) => {
                                msg_events.write(msg_event);
                                cmd_events.clear();
                                return;
                            }
                            #[cfg(feature = "bot")]
                            CmdSource::Discord(_) => {
                                msg_events.write(msg_event);
                                cmd_events.clear();
                                return;
                            }
                            CmdSource::Minecraft(_) => local_settings,
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
            let Some((chamber, _distance)) = player_chambers
                .filter(|(_, distance)| *distance <= global_settings.pearl_view_distance * 4)
                .min_by_key(|(chamber, distance)| {
                    let shared_count = stasis_chambers
                        .0
                        .values()
                        .filter(|c| c.block_pos == chamber.block_pos)
                        .filter(|c| c.owner_uuid != chamber.owner_uuid)
                        .count();

                    // First compare by shared count, then by distance
                    (shared_count, *distance)
                })
            else {
                let location = &local_settings.auto_pearl.location;
                msg_event.content = format!("Pearl not found at {location}");
                msg_event.status = 404;
                msg_events.write(msg_event);
                cmd_events.clear();
                return;
            };

            msg_event.status = 200;
            msg_event.content = match count {
                0 => str!("I'm on my way, this was your last pearl!"),
                1 => str!("I'm on my way, you have one more pearl!"),
                c => format!("I'm on my way, you have {c} more pearls."),
            };

            msg_events.write(msg_event);
            pearl_events.write(PearlGotoEvent(PearlEvent {
                entity,
                idle_goal: local_settings.auto_pearl.idle_goal.clone(),
                block_pos: chamber.block_pos,
                owner_uuid: chamber.owner_uuid,
            }));

            cmd_events.clear();
            return;
        }
    }
}
