use std::time::Duration;

use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::prelude::*;

/// View players first and last seen. <https://2b2t.vc>
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SeenCommandPlugin;

impl ChatCmd for SeenCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["seen"]
    }
}

impl Plugin for SeenCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_seen_command_events
                .ambiguous_with_all()
                .before(MinecraftChatPlugin::handle_send_whisper_events)
                .after(MinecraftChatPlugin::handle_chat_received_events),
        );
    }
}

impl SeenCommandPlugin {
    pub fn handle_seen_command_events(
        mut command_events: EventReader<CommandEvent>,
        mut whisper_events: EventWriter<WhisperEvent>,
    ) {
        if let Some(event) = command_events.read().next() {
            let ChatCmds::Seen(_plugin) = event.command else {
                return;
            };

            let mut whisper_event = WhisperEvent {
                content: String::new(),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            let Some(player_name) = event.args.iter().next() else {
                whisper_event.content = str!("Missing player name");
                whisper_event.status = 400;
                whisper_events.send(whisper_event);
                return;
            };

            let response = match ureq::get("https://api.2b2t.vc/seen")
                .query("playerName", player_name)
                .timeout(Duration::from_secs(25))
                .call()
            {
                Ok(response) => response,
                Err(error) => {
                    whisper_event.content = format!("Error: {error}");
                    whisper_event.status = 500;
                    whisper_events.send(whisper_event);
                    error!("{error}");
                    return;
                }
            };

            if response.status() == 204 {
                whisper_event.content = format!("Player not found: {player_name}");
                whisper_event.status = 204;
                whisper_events.send(whisper_event);
                return;
            }

            let Ok(json) = response.into_json::<Json>() else {
                whisper_event.content = str!("Failed to parse JSON");
                whisper_event.status = 500;
                whisper_events.send(whisper_event);
                return;
            };

            let (Some(first), Some(last)) = (json.first_seen, json.last_seen) else {
                whisper_event.content = str!("Player has never joined");
                whisper_event.status = 200;
                whisper_events.send(whisper_event);
                return;
            };

            whisper_event.content = format!(
                "First: {} | Last: {}",
                first.format("%Y-%m-%d"),
                last.format("%Y-%m-%d %H:%M")
            );
            whisper_events.send(whisper_event);
        }

        command_events.clear();
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    first_seen: Option<DateTime<Utc>>,
    last_seen:  Option<DateTime<Utc>>,
}
