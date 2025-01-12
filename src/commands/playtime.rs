use std::time::Duration;

use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
};
use serde::Deserialize;

use crate::prelude::*;

/// View players play time. <https://2b2t.vc>
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlaytimeCommandPlugin;

impl ChatCmd for PlaytimeCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["playtime"]
    }
}

impl Plugin for PlaytimeCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_playtime_command_events
                .ambiguous_with_all()
                .before(DiscordChatPlugin::handle_send_whisper_events)
                .before(MinecraftChatPlugin::handle_send_whisper_events)
                .after(MinecraftChatPlugin::handle_chat_received_events),
        );
    }
}

impl PlaytimeCommandPlugin {
    pub fn handle_playtime_command_events(
        mut command_events: EventReader<CommandEvent>,
        mut whisper_events: EventWriter<WhisperEvent>,
    ) {
        if let Some(event) = command_events.read().next() {
            let ChatCmds::Playtime(_plugin) = event.command else {
                return;
            };

            let mut whisper_event = WhisperEvent {
                entity:  event.entity,
                source:  event.source,
                sender:  event.sender,
                content: String::new(),
            };

            let Some(player_name) = event.args.iter().next() else {
                whisper_event.content = str!("[400] Missing player name");
                whisper_events.send(whisper_event);
                return;
            };

            let response = match ureq::get("https://api.2b2t.vc/playtime")
                .query("playerName", player_name)
                .timeout(Duration::from_secs(25))
                .call()
            {
                Ok(response) => response,
                Err(error) => {
                    whisper_event.content = format!("[404] Player not found: {player_name}");
                    whisper_events.send(whisper_event);
                    error!("{error}");
                    return;
                }
            };

            if response.status() == 204 {
                whisper_event.content = str!("[204] Invalid player?");
                whisper_events.send(whisper_event);
                return;
            }

            let Ok(json) = response.into_json::<Json>() else {
                whisper_event.content = str!("[500] Failed to parse JSON");
                whisper_events.send(whisper_event);
                return;
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

        command_events.clear();
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    playtime_seconds: u64,
}
