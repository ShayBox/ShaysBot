use std::time::Duration;

use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::plugins::commands::prelude::*;

/// 2B2T Seen Command <https://2b2t.vc>
pub struct SeenCommandPlugin;

impl Plugin for SeenCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, handle_seen_register).add_systems(
            Update,
            handle_seen_command_event
                .ambiguous_with_all()
                .before(handle_discord_whisper_event)
                .before(handle_minecraft_whisper_event)
                .after(handle_chat_received_event),
        );
    }
}

pub fn handle_seen_register(mut registry: ResMut<Registry>) {
    registry.register("seen", Command::Seen);
}

pub fn handle_seen_command_event(
    mut command_events: EventReader<CommandEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
) {
    for event in command_events.read() {
        if event.command != Command::Seen {
            continue;
        }

        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            source:  event.source.clone(),
            sender:  event.sender.clone(),
            content: String::new(),
        };

        let Some(player_name) = event.args.iter().next() else {
            whisper_event.content = String::from("[400] Missing player name");
            whisper_events.send(whisper_event);
            continue;
        };

        let response = match ureq::get("https://api.2b2t.vc/seen")
            .query("playerName", player_name)
            .timeout(Duration::from_secs(25))
            .call()
        {
            Ok(response) => response,
            Err(error) => {
                whisper_event.content = format!("[500] Error: {error}");
                whisper_events.send(whisper_event);
                error!("{error}");
                continue;
            }
        };

        if response.status() == 204 {
            whisper_event.content = format!("[204] Player not found: {player_name}");
            whisper_events.send(whisper_event);
            continue;
        }

        let Ok(json) = response.into_json::<Json>() else {
            whisper_event.content = String::from("[500] Failed to parse JSON");
            whisper_events.send(whisper_event);
            continue;
        };

        let (Some(first), Some(last)) = (json.first_seen, json.last_seen) else {
            whisper_event.content = String::from("[200] Player has never joined");
            whisper_events.send(whisper_event);
            continue;
        };

        whisper_event.content = format!(
            "First: {} | Last: {}",
            first.format("%Y-%m-%d"),
            last.format("%Y-%m-%d %H:%M")
        );
        whisper_events.send(whisper_event);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    first_seen: Option<DateTime<Utc>>,
    last_seen:  Option<DateTime<Utc>>,
}
