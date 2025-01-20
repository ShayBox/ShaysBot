use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
};

use crate::prelude::*;

/// Connect a bot to the Minecraft server.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JoinCommandPlugin;

impl ChatCmd for JoinCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["connect", "join", "c"]
    }
}

impl Plugin for JoinCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_join_command_events
                .ambiguous_with_all()
                .before(MinecraftChatPlugin::handle_send_whisper_events)
                .after(MinecraftChatPlugin::handle_chat_received_events),
        );
    }
}

impl JoinCommandPlugin {
    pub fn handle_join_command_events(
        mut command_events: EventReader<CommandEvent>,
        mut whisper_events: EventWriter<WhisperEvent>,
        swarm_state: Res<SwarmState>,
    ) {
        if let Some(event) = command_events.read().next() {
            let ChatCmds::Join(_plugin) = event.command else {
                return;
            };

            let mut whisper_event = WhisperEvent {
                content: String::new(),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            let Some(bot_name) = event.args.iter().next() else {
                whisper_event.content = str!("[404] Missing bot name");
                whisper_event.status = 404;
                whisper_events.send(whisper_event);
                return;
            };

            whisper_event.content = format!("[202] Enabling AutoReconnect for {bot_name}");
            whisper_event.status = 202;
            whisper_events.send(whisper_event);
            swarm_state
                .auto_reconnect
                .write()
                .insert(bot_name.to_lowercase(), (true, 0));
        }

        command_events.clear();
    }
}
