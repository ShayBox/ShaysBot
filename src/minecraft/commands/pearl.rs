use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
    entity::Position,
    BlockPos,
    TabList,
};

use crate::{
    minecraft::{
        commands::{handle_whisper_event, Command, CommandEvent, Commands, WhisperEvent},
        prelude::*,
    },
    trapdoors::TrapdoorsLock,
};

pub struct PearlCommandPlugin;

impl Plugin for PearlCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, handle_startup).add_systems(
            Update,
            handle_command_event
                .before(handle_pearl_event)
                .before(handle_whisper_event),
        );
    }
}

pub fn handle_startup(mut commands: ResMut<Commands>) {
    for alias in ["pearl", "tp", "teleport", "pull", "here", "home", "warp"] {
        commands.0.insert(String::from(alias), Command::Pearl);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_command_event(
    mut events: EventReader<CommandEvent>,
    mut pearl_events: EventWriter<PearlEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
    query: Query<(&TabList, &TrapdoorsLock, &Position)>,
) {
    for event in events.read() {
        let Ok((tab_list, trapdoors, position)) = query.get(event.entity) else {
            continue;
        };

        if event.command != Command::Pearl {
            continue;
        }

        let sender = &event.sender;
        let Some(uuid) = tab_list
            .iter()
            .find(|(_, info)| &info.profile.name == sender)
            .map(|(uuid, _)| uuid)
        else {
            whisper_events.send(WhisperEvent {
                entity:  event.entity,
                sender:  sender.clone(),
                content: String::from("I couldn't find you in my list"),
            });

            continue;
        };

        let trapdoors = trapdoors.0.read().0.clone();
        let Some(trapdoor) = trapdoors
            .clone()
            .into_values()
            .filter(|trapdoor| &trapdoor.owner_uuid == uuid)
            .min_by_key(|trapdoor| {
                let shared_count = trapdoors
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
            whisper_events.send(WhisperEvent {
                entity:  event.entity,
                sender:  sender.clone(),
                content: String::from("[404] Pearl not found."),
            });

            continue;
        };

        whisper_events.send(WhisperEvent {
            entity:  event.entity,
            sender:  sender.clone(),
            content: String::from("[202] I'm on my way!"),
        });

        pearl_events.send(PearlEvent {
            entity:    event.entity,
            block_pos: trapdoor.block_pos,
        });
    }
}
