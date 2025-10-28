use std::time::Duration;

use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use ureq::{Agent, config::Config};

use crate::prelude::*;

/// Fetch a players first and last seen time using <https://2b2t.vc>.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SeenCommandPlugin;

impl Cmd for SeenCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["seen"]
    }
}

impl Plugin for SeenCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_seen_cmd_events
                .ambiguous_with_all()
                .before(MinecraftParserPlugin::handle_send_msg_events)
                .after(MinecraftParserPlugin::handle_chat_received_events),
        );
    }
}

impl SeenCommandPlugin {
    pub fn handle_seen_cmd_events(
        mut cmd_events: MessageReader<CmdEvent>,
        mut msg_events: MessageWriter<MsgEvent>,
    ) {
        if let Some(event) = cmd_events.read().next() {
            let Cmds::Seen(_plugin) = event.cmd else {
                return;
            };

            let mut msg_event = MsgEvent {
                content: String::new(),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            let Some(player_name) = event.args.iter().next() else {
                msg_event.content = str!("Missing player name");
                msg_event.status = 400;
                msg_events.write(msg_event);
                return;
            };

            let timeout = Some(Duration::from_secs(25));
            let config = Config::builder().timeout_global(timeout).build();
            let agent = Agent::from(config);
            let mut response = match agent
                .get("https://api.2b2t.vc/seen")
                .query("playerName", player_name)
                .call()
            {
                Ok(response) => response,
                Err(error) => {
                    msg_event.content = format!("Error: {error}");
                    msg_event.status = 500;
                    msg_events.write(msg_event);
                    error!("{error}");
                    return;
                }
            };

            if response.status() == 204 {
                msg_event.content = format!("Player not found: {player_name}");
                msg_event.status = 204;
                msg_events.write(msg_event);
                return;
            }

            let Ok(json) = response.body_mut().read_json::<Json>() else {
                msg_event.content = str!("Failed to parse JSON");
                msg_event.status = 500;
                msg_events.write(msg_event);
                return;
            };

            let (Some(first), Some(last)) = (json.first_seen, json.last_seen) else {
                msg_event.content = str!("Player has never joined");
                msg_event.status = 200;
                msg_events.write(msg_event);
                return;
            };

            msg_event.content = format!(
                "First: {} | Last: {}",
                first.format("%Y-%m-%d"),
                last.format("%Y-%m-%d %H:%M")
            );
            msg_events.write(msg_event);
        }

        cmd_events.clear();
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    first_seen: Option<DateTime<Utc>>,
    last_seen:  Option<DateTime<Utc>>,
}
