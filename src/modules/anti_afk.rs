use azalea::{
    app::{App, Plugin, Startup},
    ecs::prelude::*,
    interact::{handle_swing_arm_event, SwingArmEvent},
    mining::continue_mining_block,
    packet::game::SendPacketEvent,
    prelude::*,
    protocol::packets::game::{ServerboundClientInformation, ServerboundGamePacket},
    ClientInformation,
};

use crate::prelude::*;

/// Automatically swing arm to avoid being kicked
pub struct AntiAfkPlugin;

impl Plugin for AntiAfkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::change_view_distance)
            .add_systems(
                GameTick,
                Self::handle_anti_afk
                    .before(handle_swing_arm_event)
                    .after(continue_mining_block)
                    .after(GameTickPlugin::handle_game_ticks),
            );
    }
}

impl AntiAfkPlugin {
    pub fn change_view_distance(mut query: Query<(&mut ClientInformation, &LocalSettings)>) {
        for (mut client_information, local_settings) in &mut query {
            client_information.view_distance = local_settings.anti_afk.view_distance;
        }
    }

    pub fn handle_anti_afk(
        mut query: Query<(Entity, &LocalSettings, &GameTicks, &mut ClientInformation)>,
        mut send_packet_events: EventWriter<SendPacketEvent>,
        mut swing_arm_events: EventWriter<SwingArmEvent>,
    ) {
        for (entity, local_settings, game_ticks, mut client_information) in &mut query {
            if !local_settings.anti_afk.enabled {
                continue;
            }

            if game_ticks.0 % local_settings.anti_afk.delay_ticks != 0 {
                continue;
            }

            client_information.view_distance = local_settings.anti_afk.view_distance;
            send_packet_events.send(SendPacketEvent {
                sent_by: entity,
                packet:  ServerboundGamePacket::ClientInformation(ServerboundClientInformation {
                    information: client_information.clone(),
                }),
            });

            trace!("Anti-Afk Swing Arm Event");
            swing_arm_events.send(SwingArmEvent { entity });
        }
    }
}
