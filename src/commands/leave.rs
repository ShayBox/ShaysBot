use azalea::{
    app::{App, Plugin, Update},
    disconnect::DisconnectEvent,
    ecs::prelude::*,
    GameProfileComponent,
};

use crate::prelude::*;

pub const LEAVE_PREFIX: &str = "Leave Command: ";

/// Disconnect a bot from the Minecraft server.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LeaveCommandPlugin;

impl ChatCmd for LeaveCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["leave", "disconnect", "dc"]
    }
}

impl Plugin for LeaveCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_leave_command_events
                .ambiguous_with_all()
                .before(DiscordChatPlugin::handle_send_whisper_events)
                .before(MinecraftChatPlugin::handle_send_whisper_events)
                .after(MinecraftChatPlugin::handle_chat_received_events),
        );
    }
}

impl LeaveCommandPlugin {
    pub fn handle_leave_command_events(
        mut command_events: EventReader<CommandEvent>,
        mut whisper_events: EventWriter<WhisperEvent>,
        mut disconnect_events: EventWriter<DisconnectEvent>,
        query: Query<&GameProfileComponent>,
    ) {
        for event in command_events.read().cloned() {
            let ChatCmds::Leave(_plugin) = event.command else {
                return;
            };

            let Ok(profile) = query.get(event.entity) else {
                continue;
            };

            let mut whisper_event = WhisperEvent {
                entity:  event.entity,
                source:  event.source,
                sender:  event.sender,
                content: String::new(),
            };

            let Some(bot_name) = event.args.iter().next().cloned() else {
                whisper_event.content = str!("[404] Missing bot name");
                whisper_events.send(whisper_event);
                continue;
            };

            let bot_name = bot_name.to_lowercase();
            if profile.name.to_lowercase() != bot_name {
                if event.message {
                    whisper_event.content = str!("[406] Invalid bot name");
                    whisper_events.send(whisper_event);
                }
                continue; /* Not this account */
            }

            disconnect_events.send(DisconnectEvent {
                entity: event.entity,
                reason: Some(format!("{LEAVE_PREFIX}{:?}", event.sender).into()),
            });
        }

        command_events.clear();
    }
}
