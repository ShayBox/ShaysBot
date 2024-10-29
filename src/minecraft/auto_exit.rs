use azalea::{
    app::{App, Plugin, Update},
    disconnect::DisconnectEvent,
    ecs::prelude::*,
};

pub struct AutoExitPlugin;

impl Plugin for AutoExitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_disconnect_event);
    }
}

pub fn handle_disconnect_event(mut events: EventReader<DisconnectEvent>) {
    for event in events.read() {
        let Some(reason) = &event.reason else {
            continue;
        };

        println!("Disconnect Reason: {}", reason.to_ansi());
        if ["AutoDisconnect"].contains(&&*reason.to_string()) {
            eprintln!("Exiting to stay disconnected...");
            std::process::exit(1);
        }
    }
}
