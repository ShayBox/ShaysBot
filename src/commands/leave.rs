use azalea::{
    app::{App, Plugin, Update},
    disconnect::DisconnectEvent,
    ecs::prelude::*,
    GameProfileComponent,
};

use crate::prelude::*;

pub const LEAVE_PREFIX: &str = "Leave Command: ";

/// Disconnect an account from the server and disable `AutoReconnect`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LeaveCommandPlugin;

impl Cmd for LeaveCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["leave", "disconnect", "dc"]
    }
}

impl Plugin for LeaveCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_leave_cmd_events
                .ambiguous_with_all()
                .before(MinecraftParserPlugin::handle_send_msg_events)
                .after(MinecraftParserPlugin::handle_chat_received_events),
        );
    }
}

impl LeaveCommandPlugin {
    pub fn handle_leave_cmd_events(
        mut cmd_events: EventReader<CmdEvent>,
        mut msg_events: EventWriter<MsgEvent>,
        mut disconnect_events: EventWriter<DisconnectEvent>,
        query: Query<(Entity, &GameProfileComponent)>,
    ) {
        for event in cmd_events.read().cloned() {
            let Cmds::Leave(_plugin) = event.cmd else {
                return;
            };

            let mut msg_event = MsgEvent {
                content: String::new(),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            let Some(bot_name) = event.args.iter().next().cloned() else {
                msg_event.content = str!("Missing bot name");
                msg_event.status = 404;
                msg_events.write(msg_event);
                continue;
            };

            let Some((entity, profile)) = query.iter().find(|(_, p)| p.name == bot_name) else {
                msg_event.content = str!("offline bot name");
                msg_event.status = 404;
                msg_events.write(msg_event);
                continue;
            };

            let bot_name = bot_name.to_lowercase();
            if profile.name.to_lowercase() != bot_name {
                if event.message {
                    msg_event.content = str!("Invalid bot name");
                    msg_event.status = 406;
                    msg_events.write(msg_event);
                }
                continue; /* Not this account */
            }

            disconnect_events.write(DisconnectEvent {
                entity,
                reason: Some(format!("{LEAVE_PREFIX}{:?}", event.sender).into()),
            });
        }

        cmd_events.clear();
    }
}
