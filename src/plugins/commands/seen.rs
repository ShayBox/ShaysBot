use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::plugins::commands::{
    handle_chat_received_event,
    handle_whisper_event,
    Command,
    CommandEvent,
    Registry,
    WhisperEvent,
};

pub struct SeenCommandPlugin;

impl Plugin for SeenCommandPlugin {
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
    registry.register("seen", Command::Seen);
}

pub fn handle_command_event(
    mut command_events: EventReader<CommandEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
) {
    for event in command_events.read() {
        if event.command != Command::Seen {
            continue;
        }

        let mut whisper_event = WhisperEvent {
            source:  event.source,
            entity:  event.entity,
            sender:  event.sender.clone(),
            content: String::new(),
        };

        let Some(player_name) = event.args.iter().next() else {
            whisper_event.content = String::from("[400] Missing player name!");
            whisper_events.send(whisper_event);
            continue;
        };

        let request = ureq::get("https://api.2b2t.vc/seen").query("playerName", player_name);
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

        whisper_event.content = format!(
            "First: {} | Last: {}",
            json.first_seen.format("%Y-%m-%d"),
            json.last_seen.format("%Y-%m-%d %H:%M")
        );
        whisper_events.send(whisper_event);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    first_seen: DateTime<Utc>,
    last_seen:  DateTime<Utc>,
}
