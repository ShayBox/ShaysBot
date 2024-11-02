use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
    entity::Position,
    BlockPos,
    TabList,
};

use crate::{
    plugins::{
        commands::{
            handle_chat_received_event,
            handle_whisper_event,
            Command,
            CommandEvent,
            Registry,
            WhisperEvent,
        },
        prelude::*,
    },
    trapdoors::Trapdoors,
};

/// Pearl Stasis Command
pub struct PearlCommandPlugin;

impl Plugin for PearlCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, handle_register).add_systems(
            Update,
            handle_pearl_command_event
                .ambiguous_with_all()
                .before(handle_pearl_goto_event)
                .before(handle_whisper_event)
                .after(handle_chat_received_event),
        );
    }
}

pub fn handle_register(mut registry: ResMut<Registry>) {
    for alias in ["pearl", "tp", "teleport", "pull", "here", "home", "warp"] {
        registry.register(alias, Command::Pearl);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_pearl_command_event(
    mut command_events: EventReader<CommandEvent>,
    mut pearl_events: EventWriter<PearlGotoEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
    query: Query<(&TabList, &Position)>,
    trapdoors: Res<Trapdoors>,
) {
    for event in command_events.read() {
        let Ok((tab_list, position)) = query.get(event.entity) else {
            continue;
        };

        if event.command != Command::Pearl {
            continue;
        }

        let sender = &event.sender;
        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            source:  event.source.clone(),
            sender:  event.sender.clone(),
            content: String::new(),
        };

        if sender.is_empty() {
            whisper_event.content = String::from("[404] Missing Sender!");
            whisper_events.send(whisper_event);
            continue;
        }

        let Some(uuid) = tab_list
            .iter()
            .find(|(_, info)| &info.profile.name == sender)
            .map(|(uuid, _)| uuid)
        else {
            whisper_event.content = format!("[404] Sender not found: {sender}");
            whisper_events.send(whisper_event);
            continue;
        };

        let Some(trapdoor) = trapdoors
            .0
            .clone()
            .into_values()
            .filter(|trapdoor| &trapdoor.owner_uuid == uuid)
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
            whisper_event.content = String::from("[404] Pearl not found.");
            whisper_events.send(whisper_event);
            continue;
        };

        whisper_event.content = String::from("[202] I'm on my way!");
        whisper_events.send(whisper_event);

        pearl_events.send(PearlGotoEvent {
            entity:     event.entity,
            block_pos:  trapdoor.block_pos,
            owner_uuid: trapdoor.owner_uuid,
        });
    }
}
