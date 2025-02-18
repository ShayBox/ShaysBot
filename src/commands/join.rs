use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
};

use crate::prelude::*;

/// Connect an account to the server by enabling `AutoReconnect`
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JoinCommandPlugin;

impl Cmd for JoinCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["connect", "join", "c"]
    }
}

impl Plugin for JoinCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_join_cmd_events
                .ambiguous_with_all()
                .before(MinecraftParserPlugin::handle_send_msg_events)
                .after(MinecraftParserPlugin::handle_chat_received_events),
        );
    }
}

impl JoinCommandPlugin {
    pub fn handle_join_cmd_events(
        mut cmd_events: EventReader<CmdEvent>,
        mut msg_events: EventWriter<MsgEvent>,
        swarm_state: Res<SwarmState>,
    ) {
        if let Some(event) = cmd_events.read().next() {
            let Cmds::Join(_plugin) = event.cmd else {
                return;
            };

            let mut msg_event = MsgEvent {
                content: String::new(),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            let Some(bot_name) = event.args.iter().next() else {
                msg_event.content = str!("[404] Missing bot name");
                msg_event.status = 404;
                msg_events.send(msg_event);
                return;
            };

            msg_event.content = format!("[202] Enabling AutoReconnect for {bot_name}");
            msg_event.status = 202;
            msg_events.send(msg_event);
            swarm_state
                .auto_reconnect
                .write()
                .insert(bot_name.to_lowercase(), (true, 0));
        }

        cmd_events.clear();
    }
}
