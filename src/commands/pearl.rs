use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::Position,
    BlockPos,
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

pub fn handle_pearl_command_event(
    mut command_events: EventReader<CommandEvent>,
    mut pearl_events: EventWriter<PearlGotoEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
    query: Query<(&TabList, &Position)>,
    settings: Res<Settings>,
    trapdoors: Res<Trapdoors>,
) {
    for mut event in command_events.read().cloned() {
        let Commands::Pearl(_plugin) = event.command else {
            continue;
        };

        let Ok((tab_list, position)) = query.get(event.entity) else {
            continue;
        };

        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            source:  event.source,
            sender:  event.sender.clone(),
            content: String::new(),
        };

        let uuid = match event.sender {
            CommandSender::Minecraft(username) => username,
            CommandSender::Discord(user_id) => {
                let Some(username) = event.args.pop_front() else {
                    whisper_event.content = String::from("[404] Missing username");
                    whisper_events.send(whisper_event);
                    continue;
                };

                let Some((uuid, _info)) = tab_list
                    .iter()
                    .find(|(_, info)| info.profile.name == username)
                else {
                    whisper_event.content = format!("[404] {username} is not online");
                    whisper_events.send(whisper_event);
                    continue;
                };

                if !settings.whitelisted.is_empty() {
                    let Some(whitelist) = settings.whitelisted.get(uuid) else {
                        continue;
                    };

                    let Some(discord_id) = whitelist else {
                        whisper_event.content = String::from("[404] Link not found");
                        whisper_events.send(whisper_event);
                        continue;
                    };

                    if discord_id != &user_id.to_string() {
                        whisper_event.content = String::from("[403] Not your account");
                        whisper_events.send(whisper_event);
                        continue;
                    }
                }

                uuid.clone()
            }
        };

        let Some(trapdoor) = trapdoors
            .0
            .clone()
            .into_values()
            .filter(|trapdoor| trapdoor.owner_uuid == uuid)
            .min_by_key(|trapdoor| {
                let shared_count = trapdoors
                    .0
                    .values()
                    .filter(|td| td.block_pos == trapdoor.block_pos)
                    .filter(|td| td.owner_uuid != trapdoor.owner_uuid)
                    .count();

                let client_pos = BlockPos::from(position);
                let distance = (client_pos.x - trapdoor.block_pos.x).abs()
                    + (client_pos.y - trapdoor.block_pos.y).abs()
                    + (client_pos.z - trapdoor.block_pos.z).abs();

                // First compare by shared count, then by distance
                (shared_count, distance)
            })
        else {
            whisper_event.content = String::from("[404] Pearl not found");
            whisper_events.send(whisper_event);
            continue;
        };

        whisper_event.content = String::from("[202] I'm on my way");
        whisper_events.send(whisper_event);

        pearl_events.send(PearlGotoEvent {
            entity:     event.entity,
            block_pos:  trapdoor.block_pos,
            owner_uuid: trapdoor.owner_uuid,
        });
    }
}
