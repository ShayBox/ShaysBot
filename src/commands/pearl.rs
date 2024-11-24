use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::Position,
    BlockPos,
    GameProfileComponent,
    TabList,
};
use handlers::prelude::*;

use crate::{
    commands::{prelude::*, Commands},
    plugins::prelude::*,
    trapdoors::Trapdoors,
    Settings,
};

/// Pearl Stasis Command
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PearlCommandPlugin;

impl Command for PearlCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["pearl", "tp", "teleport", "warp", "home"]
    }
}

impl Plugin for PearlCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_pearl_command_event
                .ambiguous_with_all()
                .before(handle_pearl_goto_event)
                .before(handle_discord_whisper_event)
                .before(handle_minecraft_whisper_event)
                .after(handle_chat_received_event),
        );
    }
}

#[allow(clippy::too_many_lines)]
pub fn handle_pearl_command_event(
    mut command_events: EventReader<CommandEvent>,
    mut pearl_events: EventWriter<PearlGotoEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
    query: Query<(&TabList, &Position, &GameProfileComponent)>,
    settings: Res<Settings>,
    trapdoors: Res<Trapdoors>,
) {
    for mut event in command_events.read().cloned() {
        let Commands::Pearl(_plugin) = event.command else {
            continue;
        };

        let Ok((tab_list, position, profile)) = query.get(event.entity) else {
            continue;
        };

        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            source:  event.source,
            sender:  event.sender,
            content: String::new(),
        };

        whisper_event.content = format!(
            "[404] Invalid or Missing location, Available location(s): {}",
            settings
                .locations
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );

        let uuid = match event.sender {
            CommandSender::Minecraft(username) => username,
            CommandSender::Discord(user_id) => {
                let Some(username) = event.args.pop_front() else {
                    whisper_event.content = str!("[404] Missing username");
                    whisper_events.send(whisper_event);
                    command_events.clear();
                    return;
                };

                let Some((uuid, _info)) = tab_list
                    .iter()
                    .find(|(_, info)| info.profile.name == username)
                else {
                    whisper_event.content = format!("[404] {username} is not online");
                    whisper_events.send(whisper_event);
                    command_events.clear();
                    return;
                };

                if settings.whitelist {
                    let Some(whitelist) = settings.whitelisted.get(uuid) else {
                        command_events.clear();
                        return; /* Not Whitelisted */
                    };

                    let Some(discord_id) = whitelist else {
                        whisper_event.content = str!("[404] Link not found");
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    };

                    if discord_id != &user_id.to_string() {
                        whisper_event.content = str!("[403] Not your account");
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    }
                }

                *uuid
            }
        };

        let (alias, bot) = match event.args.pop_front() {
            Some(alias) => match settings.locations.get_key_value(&alias) {
                Some((alias, bot_settings)) if bot_settings.account_username == profile.name => {
                    (alias, bot_settings)
                }
                Some(_) => {
                    whisper_event.content = str!("[500] I'm not at that location");
                    whisper_events.send(whisper_event);
                    continue;
                }
                _ => {
                    whisper_events.send(whisper_event);
                    command_events.clear();
                    return;
                }
            },
            None => match event.source {
                CommandSource::Discord(_) => match settings.locations.iter().next() {
                    Some(location) if settings.locations.len() == 1 => location,
                    _ => {
                        whisper_events.send(whisper_event);
                        command_events.clear();
                        return;
                    }
                },
                CommandSource::Minecraft(_) => match settings
                    .locations
                    .iter()
                    .find(|l| l.1.account_username == profile.name)
                {
                    Some((alias, bot)) if bot.account_username == profile.name => (alias, bot),
                    _ => {
                        continue;
                    }
                },
            },
        };

        let client_pos = BlockPos::from(position);
        let player_trapdoors = trapdoors
            .0
            .values()
            .copied()
            .filter(|trapdoor| trapdoor.owner_uuid == uuid)
            .map(|trapdoor| {
                let distance = 1
                    + (client_pos.x - trapdoor.block_pos.x).abs()
                    + (client_pos.y - trapdoor.block_pos.y).abs()
                    + (client_pos.z - trapdoor.block_pos.z).abs();

                (trapdoor, distance)
            })
            .filter(|(_, distance)| *distance < settings.pearl_view_distance * 5);

        let count = player_trapdoors.clone().count().saturating_sub(1);
        let Some((trapdoor, _)) = player_trapdoors.min_by_key(|(trapdoor, distance)| {
            let shared_count = trapdoors
                .0
                .values()
                .filter(|td| td.block_pos == trapdoor.block_pos)
                .filter(|td| td.owner_uuid != trapdoor.owner_uuid)
                .count();

            // First compare by shared count, then by distance
            (shared_count, *distance)
        }) else {
            whisper_event.content = format!("[404] Pearl not found at {alias}");
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

        pearl_events.send(PearlGotoEvent {
            entity:     event.entity,
            idle_goal:  bot.idle_goal.clone(),
            block_pos:  trapdoor.block_pos,
            owner_uuid: trapdoor.owner_uuid,
        });

        command_events.clear();
        return;
    }
}
