use anyhow::Result;
use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    interact::BlockInteractEvent,
    BlockPos,
};

use crate::prelude::*;

/// Send an interact packet for an x, y, and z
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InteractCommandPlugin;

impl Cmd for InteractCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["interact"]
    }
}

impl Plugin for InteractCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_interact_cmd_events
                .ambiguous_with_all()
                .before(MinecraftParserPlugin::handle_send_msg_events)
                .after(MinecraftParserPlugin::handle_chat_received_events),
        );
    }
}

// TODO: Add separate AutoInteract plugin to path find first
// TODO: Add separate Interaction config to store named locations
impl InteractCommandPlugin {
    pub fn handle_interact_cmd_events(
        mut cmd_events: EventReader<CmdEvent>,
        mut msg_events: EventWriter<MsgEvent>,
        mut block_interact_events: EventWriter<BlockInteractEvent>,
    ) {
        for event in cmd_events.read().cloned() {
            let (Cmds::Interact(_plugin), Some(entity)) = (event.cmd, event.entity) else {
                continue;
            };

            let mut msg_event = MsgEvent {
                content: str!("Interacted"),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            if event.args.len() < 3 {
                msg_event.content = str!("Missing coordinate");
                msg_events.write(msg_event);
                return;
            }

            let position = if let Ok(coords) = event
                .args
                .range(0..3)
                .map(|s| s.parse::<i32>())
                .collect::<Result<Vec<_>, _>>()
            {
                BlockPos::new(coords[0], coords[1], coords[2])
            } else {
                msg_event.content = str!("Invalid coordinate");
                msg_events.write(msg_event);
                return;
            };

            block_interact_events.write(BlockInteractEvent { entity, position });
            msg_events.write(msg_event);
        }

        cmd_events.clear();
    }
}
