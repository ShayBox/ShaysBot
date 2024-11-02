use std::time::Duration;

use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
};
use serde::Deserialize;

use crate::plugins::commands::{
    handle_chat_received_event,
    handle_whisper_event,
    Command,
    CommandEvent,
    Registry,
    WhisperEvent,
};

/// 2B2T Playtime Command <https://2b2t.vc>
pub struct PlaytimeCommandPlugin;

impl Plugin for PlaytimeCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, handle_register).add_systems(
            Update,
            handle_playtime_command_event
                .ambiguous_with_all()
                .before(handle_whisper_event)
                .after(handle_chat_received_event),
        );
    }
}

pub fn handle_register(mut registry: ResMut<Registry>) {
    registry.register("playtime", Command::Playtime);
}

pub fn handle_playtime_command_event(
    mut command_events: EventReader<CommandEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
) {
    for event in command_events.read() {
        if event.command != Command::Playtime {
            continue;
        }

        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            source:  event.source.clone(),
            sender:  event.sender.clone(),
            content: String::new(),
        };

        let Some(player_name) = event.args.iter().next() else {
            whisper_event.content = String::from("[400] Missing player name!");
            whisper_events.send(whisper_event);
            continue;
        };

        let response = match ureq::get("https://api.2b2t.vc/playtime")
            .query("playerName", player_name)
            .timeout(Duration::from_secs(25))
            .call()
        {
            Ok(response) => response,
            Err(error) => {
                whisper_event.content = String::from("[404] Player not found.");
                whisper_events.send(whisper_event);
                error!("{error}");
                continue;
            }
        };

        if response.status() == 204 {
            whisper_event.content = String::from("[204] Invalid player?");
            whisper_events.send(whisper_event);
            continue;
        }

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
