use std::time::Duration;

use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        change_detection::ResMut,
        event::{EventReader, EventWriter},
        prelude::IntoSystemConfigs,
    },
};
use serde::Deserialize;

use crate::minecraft::commands::{
    handle_chat_received_event,
    handle_whisper_event,
    Command,
    CommandEvent,
    Registry,
    WhisperEvent,
};

pub struct PlaytimeCommandPlugin;

impl Plugin for PlaytimeCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, handle_register).add_systems(
            Update,
            handle_command_event
                .ambiguous_with_all()
                .before(handle_whisper_event)
                .after(handle_chat_received_event),
        );
    }
}

pub fn handle_register(mut registry: ResMut<Registry>) {
    registry.register("playtime", Command::Playtime);
}

pub fn handle_command_event(
    mut events: EventReader<CommandEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
) {
    for event in events.read() {
        if event.command != Command::Playtime {
            continue;
        }

        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            sender:  event.sender.clone(),
            content: String::new(),
        };

        let Some(player_name) = event.args.iter().next() else {
            whisper_event.content = String::from("[400] Missing player name!");
            whisper_events.send(whisper_event);
            continue;
        };

        let request = ureq::get("https://api.2b2t.vc/playtime").query("playerName", player_name);
        let response = match request.call() {
            Ok(response) => response,
            Err(error) => {
                whisper_event.content = String::from("[404] Player not found.");
                whisper_events.send(whisper_event);
                eprintln!("{error}");
                continue;
            }
        };

        let Ok(json) = response.into_json::<Json>() else {
            whisper_event.content = String::from("[500] Failed to parse JSON");
            whisper_events.send(whisper_event);
            continue;
        };

        let duration = Duration::new(json.playtime_seconds, 0);
        whisper_event.content = format!(
            "{:02}:{:02}:{:02}",
            duration.as_secs() / 3600 % 24,
            duration.as_secs() / 60 % 60,
            duration.as_secs() % 60
        );
        whisper_events.send(whisper_event);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    playtime_seconds: u64,
}
